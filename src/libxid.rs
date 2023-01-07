//! from https://github.com/jeromer/libxid, modified for (shitty) windows support
//! (This is a port of [Olivier Poitrey]'s [xid] Go library)
//!
//! Package xid is a globally unique id generator library, ready to be used safely directly in your server code.
//!
//! Xid is using Mongo Object ID algorithm to generate globally unique ids with a different serialization (base64) to make it shorter when transported as a string:
//! https://docs.mongodb.org/manual/reference/object-id/
//!
//! - 4-byte value representing the seconds since the Unix epoch,
//! - 3-byte machine identifier,
//! - 2-byte process id, and
//! - 3-byte counter, starting with a random value.
//!
//! The binary representation of the id is compatible with Mongo 12 bytes Object IDs.
//! The string representation is using base32 hex (w/o padding) for better space efficiency
//! when stored in that form (20 bytes). The hex variant of base32 is used to retain the
//! sortable property of the id.
//!
//! Xid doesn't use base64 because case sensitivity and the 2 non alphanum chars may be an
//! issue when transported as a string between various systems. Base36 wasn't retained either
//! because 1/ it's not standard 2/ the resulting size is not predictable (not bit aligned)
//! and 3/ it would not remain sortable. To validate a base32 `xid`, expect a 20 chars long,
//! all lowercase sequence of `a` to `v` letters and `0` to `9` numbers (`[0-9a-v]{20}`).
//!
//! UUIDs are 16 bytes (128 bits) and 36 chars as string representation. Twitter Snowflake
//! ids are 8 bytes (64 bits) but require machine/data-center configuration and/or central
//! generator servers. xid stands in between with 12 bytes (96 bits) and a more compact
//! URL-safe string representation (20 chars). No configuration or central generator server
//! is required so it can be used directly in server's code.
//!
//! | Name        | Binary Size | String Size    | Features
//! |-------------|-------------|----------------|----------------
//! | [UUID]      | 16 bytes    | 36 chars       | configuration free, not sortable
//! | [shortuuid] | 16 bytes    | 22 chars       | configuration free, not sortable
//! | [Snowflake] | 8 bytes     | up to 20 chars | needs machin/DC configuration, needs central server, sortable
//! | [MongoID]   | 12 bytes    | 24 chars       | configuration free, sortable
//! | xid         | 12 bytes    | 20 chars       | configuration free, sortable
//!
//! [UUID]: https://en.wikipedia.org/wiki/Universally_unique_identifier
//! [shortuuid]: https://github.com/stochastic-technologies/shortuuid
//! [Snowflake]: https://blog.twitter.com/2010/announcing-snowflake
//! [MongoID]: https://docs.mongodb.org/manual/reference/object-id/
//!
//! Features:
//!
//! - Size: 12 bytes (96 bits), smaller than UUID, larger than snowflake
//! - Base32 hex encoded by default (20 chars when transported as printable string, still sortable)
//! - Non configured, you don't need set a unique machine and/or data center id
//! - K-ordered
//! - Embedded time with 1 second precision
//! - Unicity guaranteed for 16,777,216 (24 bits) unique ids per second and per host/process
//! - Lock-free (i.e.: unlike UUIDv1 and v2)
//!
//! Notes:
//!
//! - Xid is dependent on the system time, a monotonic counter and so is not cryptographically secure.
//! If unpredictability of IDs is important, you should NOT use xids.
//! It is worth noting that most of the other UUID like implementations are also not cryptographically secure.
//! You shoud use libraries that rely on cryptographically secure sources if you want a truly random ID generator.
//!
//! References:
//!
//! - https://www.slideshare.net/davegardnerisme/unique-id-generation-in-distributed-systems
//! - https://en.wikipedia.org/wiki/Universally_unique_identifier
//! - https://blog.twitter.com/2010/announcing-snowflake
//!
//! ## Usage
//!
//! ```rust
//! use libxid;
//!
//! // initialize it once, reuse it afterwards
//! let mut g = libxid::new_generator();
//!
//! for i in 0..10{
//!     let id = g.new_id().unwrap();
//!
//!     println!(
//!             "encoded: {:?}    machine: {:?}    counter: {:?}    time: {:?}",
//!             id.encode(),
//!             id.machine(),
//!             id.counter(),
//!             id.time()
//!     );
//! }
//! ```
//!
//! [Olivier Poitrey]: https://github.com/rs
//! [xid]: https://github.com/rs/xid

extern crate byteorder;
extern crate crc32fast;
extern crate gethostname;
extern crate md5;
extern crate rand;

use byteorder::{BigEndian, ByteOrder};
use gethostname::*;
use rand::prelude::*;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::error::Error;
use std::fmt;
use std::io;
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const ID_LEN: usize = 12;

// ---

#[derive(Clone, Debug)]
pub struct IDGenerationError(String);

impl Error for IDGenerationError {
    fn description(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for IDGenerationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---

pub struct Generator {
    counter: AtomicUsize,
    machine_id: [u8; 3],
    pid: u32,
}

pub fn new_generator() -> Generator {
    Generator {
        counter: rand_int(),
        machine_id: read_machine_id(),
        pid: get_pid(),
    }
}

impl Generator {
    pub fn new_id(&self) -> Result<ID, IDGenerationError> {
        self.new_id_with_time(SystemTime::now())
    }

    pub fn new_id_with_time(&self, t: SystemTime) -> Result<ID, IDGenerationError> {
        match t.duration_since(UNIX_EPOCH) {
            Ok(n) => Ok(self.generate(n.as_secs())),
            Err(e) => Err(IDGenerationError(format!("{e}"))),
        }
    }

    fn generate(&self, ts: u64) -> ID {
        let mut buff = [0u8; ID_LEN];

        BigEndian::write_u32(&mut buff, ts as u32);

        buff[4] = self.machine_id[0];
        buff[5] = self.machine_id[1];
        buff[6] = self.machine_id[2];

        buff[7] = (self.pid >> 8) as u8;
        buff[8] = self.pid as u8;

        let i = self.counter.fetch_add(1, Ordering::SeqCst);
        buff[9] = (i >> 16) as u8;
        buff[10] = (i >> 8) as u8;
        buff[11] = (i) as u8;

        ID { val: buff }
    }
}

impl fmt::Debug for Generator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Generator {{counter: {:?}, machine_id: {:?}, pid: {:?}}}",
            self.counter, self.machine_id, self.pid
        )
    }
}

// ---

#[derive(Clone)]
pub struct ID {
    val: [u8; ID_LEN],
}

impl ID {
    pub fn encode(&self) -> String {
        let alphabet = String::from("0123456789abcdefghijklmnopqrstuv");

        let buff = alphabet.as_bytes();

        std::str::from_utf8(&[
            buff[(self.val[0] as usize) >> 3],
            buff[(self.val[1] as usize) >> 6 & 0x1F | ((self.val[0] as usize) << 2) & 0x1F],
            buff[((self.val[1] as usize) >> 1) & 0x1F],
            buff[((self.val[2] as usize) >> 4) & 0x1F | ((self.val[1] as usize) << 4) & 0x1F],
            buff[(self.val[3] as usize) >> 7 | ((self.val[2] as usize) << 1) & 0x1F],
            buff[((self.val[3] as usize) >> 2) & 0x1F],
            buff[(self.val[4] as usize) >> 5 | ((self.val[3] as usize) << 3) & 0x1F],
            buff[(self.val[4] as usize) & 0x1F],
            buff[(self.val[5] as usize) >> 3],
            buff[((self.val[6] as usize) >> 6) & 0x1F | ((self.val[5] as usize) << 2) & 0x1F],
            buff[((self.val[6] as usize) >> 1) & 0x1F],
            buff[((self.val[7] as usize) >> 4) & 0x1F | ((self.val[6] as usize) << 4) & 0x1F],
            buff[(self.val[8] as usize) >> 7 | ((self.val[7] as usize) << 1) & 0x1F],
            buff[((self.val[8] as usize) >> 2) & 0x1F],
            buff[((self.val[9] as usize) >> 5) | ((self.val[8] as usize) << 3) & 0x1F],
            buff[(self.val[9] as usize) & 0x1F],
            buff[(self.val[10] as usize) >> 3],
            buff[((self.val[11] as usize) >> 6) & 0x1F | ((self.val[10] as usize) << 2) & 0x1F],
            buff[((self.val[11] as usize) >> 1) & 0x1F],
            buff[((self.val[11] as usize) << 4) & 0x1F],
        ])
        .unwrap()
        .to_string()
    }

    pub fn decode(input: &str) -> Self {
        let mut dec = [1u8; 256];

        dec[48] = 0_u8;
        dec[49] = 1_u8;
        dec[50] = 2_u8;
        dec[51] = 3_u8;
        dec[52] = 4_u8;
        dec[53] = 5_u8;
        dec[54] = 6_u8;
        dec[55] = 7_u8;
        dec[56] = 8_u8;
        dec[57] = 9_u8;
        dec[97] = 10_u8;
        dec[98] = 11_u8;
        dec[99] = 12_u8;
        dec[100] = 13_u8;
        dec[101] = 14_u8;
        dec[102] = 15_u8;
        dec[103] = 16_u8;
        dec[104] = 17_u8;
        dec[105] = 18_u8;
        dec[106] = 19_u8;
        dec[107] = 20_u8;
        dec[108] = 21_u8;
        dec[109] = 22_u8;
        dec[110] = 23_u8;
        dec[111] = 24_u8;
        dec[112] = 25_u8;
        dec[113] = 26_u8;
        dec[114] = 27_u8;
        dec[115] = 28_u8;
        dec[116] = 29_u8;
        dec[117] = 30_u8;
        dec[118] = 31_u8;

        // XXX: the code commented below generated the array above
        // let alphabet = String::from("0123456789abcdefghijklmnopqrstuv");
        // let buff = alphabet.as_bytes();
        //
        // for i in 0..alphabet2.len() {
        //     dec[alphabet2[i] as usize] = i as u8
        // }

        let src = input.as_bytes();

        ID {
            val: [
                dec[src[0] as usize] << 3 | dec[src[1] as usize] >> 2,
                dec[src[1] as usize] << 6 | dec[src[2] as usize] << 1 | dec[src[3] as usize] >> 4,
                dec[src[3] as usize] << 4 | dec[src[4] as usize] >> 1,
                dec[src[4] as usize] << 7 | dec[src[5] as usize] << 2 | dec[src[6] as usize] >> 3,
                dec[src[6] as usize] << 5 | dec[src[7] as usize],
                dec[src[8] as usize] << 3 | dec[src[9] as usize] >> 2,
                dec[src[9] as usize] << 6 | dec[src[10] as usize] << 1 | dec[src[11] as usize] >> 4,
                dec[src[11] as usize] << 4 | dec[src[12] as usize] >> 1,
                dec[src[12] as usize] << 7
                    | dec[src[13] as usize] << 2
                    | dec[src[14] as usize] >> 3,
                dec[src[14] as usize] << 5 | dec[src[15] as usize],
                dec[src[16] as usize] << 3 | dec[src[17] as usize] >> 2,
                dec[src[17] as usize] << 6
                    | dec[src[18] as usize] << 1
                    | dec[src[19] as usize] >> 4,
            ],
        }
    }

    pub fn machine(&self) -> [u8; 3] {
        [self.val[4], self.val[5], self.val[6]]
    }

    pub fn pid(&self) -> u16 {
        BigEndian::read_u16(&[self.val[7], self.val[8]])
    }

    pub fn time(&self) -> SystemTime {
        let ts = BigEndian::read_u32(&[self.val[0], self.val[1], self.val[2], self.val[3]]);

        UNIX_EPOCH + Duration::from_secs(u64::from(ts))
    }

    pub fn counter(&self) -> u32 {
        u32::from(self.val[9]) << 16 | u32::from(self.val[10]) << 8 | (u32::from(self.val[11]))
    }
}

impl fmt::Debug for ID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ID: {:?}", self.val)
    }
}

impl fmt::Display for ID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ID: {:?}", self.encode())
    }
}

impl PartialEq for ID {
    fn eq(&self, other: &ID) -> bool {
        self.val == other.val
    }
}

impl From<&str> for ID {
    // TODO: implement try_from https://doc.rust-lang.org/std/convert/trait.TryFrom.html when no
    // longer nightly
    fn from(s: &str) -> Self {
        if s.len() == 20 {
            return ID::decode(s);
        }

        ID { val: [0u8; ID_LEN] }
    }
}

impl Eq for ID {}

impl PartialOrd for ID {
    fn partial_cmp(&self, other: &ID) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ID {
    fn cmp(&self, other: &ID) -> std::cmp::Ordering {
        self.val.cmp(&other.val)
    }
}

impl Serialize for ID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.encode().as_str())
    }
}

struct IDVisitor;

impl<'de> Visitor<'de> for IDVisitor {
    type Value = ID;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a str")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(ID::from(value))
    }
}

impl<'de> Deserialize<'de> for ID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(IDVisitor)
    }
}

// ---

fn rand_int() -> AtomicUsize {
    let mut buff = [0u8; 3];

    thread_rng().fill_bytes(&mut buff);

    let x = (buff[0] as usize) << 16 | (buff[1] as usize) << 8 | buff[2] as usize;

    AtomicUsize::new(x)
}

fn get_pid() -> u32 {
    #[allow(unused_mut)]
    let mut pid = process::id();

    // If /proc/self/cpuset exists and is not /, we can assume that we are in a
    // form of container and use the content of cpuset xor-ed with the PID in
    // order get a reasonable machine global unique PID.
    #[cfg(target_os = "linux")]
    match std::fs::read("/proc/self/cpuset") {
        Err(_) => {}

        Ok(buff) => {
            use crc32fast::Hasher;
            let mut hasher = Hasher::new();
            hasher.update(buff.as_slice());
            let checksum = hasher.finalize();

            pid ^= checksum;
        }
    }

    pid
}

fn read_machine_id() -> [u8; 3] {
    let id = match platform_machine_id() {
        // XXX: https://github.com/rust-lang/rfcs/blob/master/text/0107-pattern-guards-with-bind-by-move.md
        Ok(x) => {
            if !x.is_empty() {
                x
            } else {
                hostname()
            }
        }

        _ => hostname(),
    };

    if id.is_empty() {
        let mut buff = [0u8; 3];
        thread_rng().fill_bytes(&mut buff);
        return buff;
    }

    let hash = md5::compute(id);

    [hash[0], hash[1], hash[2]]
}

#[cfg(target_os = "linux")]
fn platform_machine_id() -> Result<String, io::Error> {
    Err(io::Error::new(io::ErrorKind::NotFound, "unsupported"))
}
#[cfg(not(target_os = "linux"))]
fn platform_machine_id() -> Result<String, io::Error> {
    Err(io::Error::new(io::ErrorKind::NotFound, "unsupported"))
}

fn hostname() -> String {
    gethostname()
        .into_string()
        .expect("can not fetch machine's hostname")
}

// ---

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_basic() {
        let total = 1e6 as u32;

        let g = new_generator();

        let mut previous_counter = 0;
        let mut previous_id = g.new_id().unwrap();

        for i in 0..total {
            let id = g.new_id().unwrap();

            assert!(
                previous_id < id,
                
                    "{} ({:?}) != {} ({:?}) {}",
                    previous_id.encode(),
                    previous_id,
                    id.encode(),
                    id,
                    i
                
            );

            if i > 0 {
                assert_eq!(id.counter(), previous_counter + 1);
            }

            previous_counter = id.counter();

            let x = id.encode();
            assert_eq!(x.len(), 20);
            assert_eq!(id.machine(), g.machine_id);

            previous_id = id;
        }
    }

    #[test]
    fn test_generation_speed() {
        let total = 1e6 as u32;

        let g = new_generator();

        let start = Instant::now();

        for _ in 0..total {
            g.new_id().unwrap();
        }

        let elapsed =
            start.elapsed().as_secs() as f64 + start.elapsed().subsec_nanos() as f64 * 1e-9;

        let limit = 0.5;

        assert!(
            elapsed <= limit,
            
                "Must generated {} ids id less than {} second, took {} seconds",
                total, limit, elapsed
            
        );
    }

    #[test]
    fn test_encoding_speed() {
        let total = 1e6 as u32;

        let g = new_generator();

        let mut buff: Vec<ID> = Vec::with_capacity(total as usize);

        for _ in 0..total {
            buff.push(g.new_id().unwrap().clone());
        }

        let start = Instant::now();

        for id in buff.into_iter() {
            id.encode();
        }

        let elapsed =
            start.elapsed().as_secs() as f64 + start.elapsed().subsec_nanos() as f64 * 1e-9;

        let limit = 1.5;

        assert!(
            elapsed <= limit,
            
                "Must encode {} ids id less than {} second, took {} seconds",
                total, limit, elapsed
            
        );
    }

    #[test]
    fn test_eq() {
        let g = new_generator();

        let a = g.new_id().unwrap();
        let b = g.new_id().unwrap();
        let c = g.new_id().unwrap();

        assert!(a == a);
        assert!(a <= a);
        assert!(a != b);
        assert!(a != c);

        assert!(a < b);
        assert!(b > a);
        assert!(b >= a);

        assert!(b < c);
        assert!(c > b);

        assert!(a < c);
        assert!(c > a);
    }

    #[test]
    fn test_from() {
        let g = new_generator();

        let a = g.new_id().unwrap();

        let b = ID::from(a.encode().as_str());

        assert_eq!(a.val, b.val);
        assert_eq!(a.encode(), b.encode());

        assert_eq!(ID::from("invalid").val, [0u8; ID_LEN]);
    }

    #[test]
    fn test_encode_decode() {
        let total = 1e6 as u32;

        let g = new_generator();

        for _ in 0..total {
            let id = g.new_id().unwrap();

            assert_eq!(id, ID::decode(&id.encode()));
        }
    }

    #[test]
    fn test_decoding_speed() {
        let total = 1e6 as u32;

        let g = new_generator();

        let mut buff: Vec<String> = Vec::with_capacity(total as usize);

        for _ in 0..total {
            let id = g.new_id().unwrap();

            buff.push(id.encode().clone());

            assert_eq!(id, ID::decode(&id.encode()));
        }

        // ----

        let start = Instant::now();

        for encoded in buff.into_iter() {
            ID::decode(&encoded);
        }

        let elapsed =
            start.elapsed().as_secs() as f64 + start.elapsed().subsec_nanos() as f64 * 1e-9;

        let limit = 0.5;

        assert!(
            elapsed <= limit,
            
                "Must decode {} ids in less than {} second, took {} seconds",
                total, limit, elapsed
            
        );
    }

    #[test]
    fn test_json() {
        let g = new_generator();

        let src = g.new_id().unwrap();

        let serialized = serde_json::to_string(&src).unwrap();

        let deserialized: ID = serde_json::from_str(&serialized).unwrap();
        assert_eq!(src, deserialized);

        let invalid: ID = serde_json::from_str("\"invalid\"").unwrap();
        assert_eq!(invalid.val, [0u8; ID_LEN]);
    }
}
