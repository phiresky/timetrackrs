// import data from App Usage android app
// https://play.google.com/store/apps/details?id=com.a0soft.gphone.uninstaller&hl=en
/*
select datetime(act.time/1000, 'unixepoch'),
case act.type
when 262 then 'Screen off (locked)'
when 2565 then 'Screen on (unlocked)'
when 200 then 'Notification posted'
when 518 then 'Screen off'
when 2309 then 'Screen on (locked)'
when 1541 then 'Screen on (unlocked B)'
when 1285 then 'Screen on (locked)'
when 7 then 'Use App'
else act.type end as type_str,
* from act
-- left join usage on usage.pid = act.pid
left join pkg on act.pid = pkg._id
where act.time > 1578873600000 and act.time < 1578956400000
order by time desc
*/
#![allow(non_snake_case)]

use crate::prelude::*;
use num_enum::TryFromPrimitive;
use rusqlite::params;
use std::path::PathBuf;

lazy_static! {
    static ref TIBU_FNAME: regex::Regex = regex::Regex::new(r#".*\.tar(\.[a-z0-9]+)?"#).unwrap();
    static ref TIBU_ENCRYPTED: regex::bytes::Regex = regex::bytes::Regex::new(
        r#"(?x)
        ^TB_ARMOR_V1\n         # magic bytes
        (?P<pass_hmac_key>[A-Za-z0-9+/]+=*)\n
        (?P<pass_hmac_result>[A-Za-z0-9+/]+=*)\n
        (?P<public_key>[A-Za-z0-9+/]+=*)\n
        (?P<enc_privkey_spec>[A-Za-z0-9+/]+=*)\n
        (?P<enc_sesskey_spec>[A-Za-z0-9+/]+=*)\n
        "#
    )
    .unwrap();
}

#[derive(StructOpt)]
pub struct AppUsageImportArgs {
    // ~/data/bck/TitaniumBackup/com.a0soft.gphone.uninstaller-20200114-030409.tar.bz2
    filename: String,
    device_name: String,
    device_type: DeviceType,
}
/*
https://github.com/phyber/TiBUdecrypter/blob/master/tibudecrypt.py
https://github.com/phyber/TiBUdecrypter/blob/master/docs/FORMAT.md

const TIBU_IV: [u8; 16] = [0u8; 16];
fn decrypt(c: regex::bytes::Captures) -> anyhow::Result<()> {
    use sha1::{Digest, Sha1};
    use crypto::buffer::{ReadBuffer, WriteBuffer};
    use hmac::{Hmac, Mac};
    let pass_hmac_key =
        base64::decode(std::str::from_utf8(c.name("pass_hmac_key").unwrap().as_bytes()).unwrap())
            .unwrap();
    let pass_hmac_result = base64::decode(
        std::str::from_utf8(c.name("pass_hmac_result").unwrap().as_bytes()).unwrap(),
    )
    .unwrap();
    let public_key =
        base64::decode(std::str::from_utf8(c.name("public_key").unwrap().as_bytes()).unwrap())
            .unwrap();
    let enc_privkey_spec = base64::decode(
        std::str::from_utf8(c.name("enc_privkey_spec").unwrap().as_bytes()).unwrap(),
    )
    .unwrap();
    let enc_sesskey_spec = base64::decode(
        std::str::from_utf8(c.name("enc_sesskey_spec").unwrap().as_bytes()).unwrap(),
    )
    .unwrap();
    println!("hmac key={:x?}", pass_hmac_key);

    let hashed_pass = {
        let mut mac = Hmac::<Sha1>::new_varkey(&pass_hmac_key).unwrap();
        let password = password.as_bytes();
        mac.input(&password);
        let res = mac.result().code();
        if &res[..] != &pass_hmac_result[..] {
            anyhow::bail!("Wrong password!");
        }
        let mut hashed_pass = Sha1::new();
        hashed_pass.input(&password);
        let hashed_pass = hashed_pass.result();
        let mut hashed_pass_padded = [0u8; 32];
        hashed_pass_padded[..20].copy_from_slice(&hashed_pass[0..20]);
        hashed_pass_padded
    };
    let dec_privkey_spec = {
        let mut decryptor = crypto::aes::cbc_decryptor(
            crypto::aes::KeySize::KeySize256,
            &hashed_pass,
            &TIBU_IV,
            crypto::blockmodes::PkcsPadding,
        );
        let mut read_buffer = crypto::buffer::RefReadBuffer::new(&enc_privkey_spec);
        let mut buffer = [0u8; 4096];
        let mut write_buffer = crypto::buffer::RefWriteBuffer::new(&mut buffer);
        let mut final_result = Vec::<u8>::new();
        loop {
            let result = decryptor
                .decrypt(&mut read_buffer, &mut write_buffer, true)
                .unwrap();
            // "write_buffer.take_read_buffer().take_remaining()" means:
            // from the writable buffer, create a new readable buffer which
            // contains all data that has been written, and then access all
            // of that data as a slice.
            final_result.extend(
                write_buffer
                    .take_read_buffer()
                    .take_remaining()
                    .iter()
                    .map(|&i| i),
            );
            match result {
                crypto::buffer::BufferResult::BufferUnderflow => break,
                crypto::buffer::BufferResult::BufferOverflow => {}
            }
        }
        final_result
    };
    println!("{:x?}", &dec_privkey_spec);
    println!("{}", dec_privkey_spec.len());
    let dec_sesskey = {
        let rsa_privkey = openssl::pkey::PKey::private_key_from_der(&dec_privkey_spec)?.rsa()?;
        let mut out = [0u8; 4096];
        let s = rsa_privkey.private_decrypt(
            &enc_sesskey_spec,
            &mut out,
            openssl::rsa::Padding::PKCS1,
        )?;
        println!("decrypted {}", s);
        &out[0..s].to_vec()
    };
}*/

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub enum DeviceType {
    Smartphone,
    Tablet,
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone, TryFromPrimitive, PartialEq)]
#[repr(i32)]
pub enum UseType {
    Unknown = 0,
    DeviceShutdown = 1,
    DeviceBoot = 2,
    UseApp = 7,
    Unknown8 = 8,
    NotificationPosted = 200,
    ScreenOffLocked = 262,
    ScreenOff = 518,
    ScreenOnLockedB = 1285,
    ScreenOnUnlockedB = 1541,
    ScreenOnLocked = 2309,
    ScreenOnUnlocked = 2565,
}

use rusqlite::types::{FromSqlResult, ValueRef};
impl rusqlite::types::FromSql for UseType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Integer(i) => UseType::try_from(i as i32).map_err(|e| {
                println!("unknown use type {}", i);
                rusqlite::types::FromSqlError::InvalidType
            }),
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}
impl std::str::FromStr for DeviceType {
    type Err = anyhow::Error;
    fn from_str(day: &str) -> anyhow::Result<Self> {
        match day {
            "Smartphone" => Ok(DeviceType::Smartphone),
            "Tablet" => Ok(DeviceType::Smartphone),
            _ => Err(anyhow::anyhow!("IDK what that is")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify)]
pub struct AppUsageEntry {
    pub device_type: DeviceType,
    pub device_name: String,
    pub act_dur: i64,
    pub act_pid: i64,
    pub act_time: i64,
    pub act_type: UseType,
    pub act_type_raw: i64,
    pub cat__id: Option<i64>,
    pub cat_cid: Option<i64>,
    pub cat_name: Option<String>,
    pub cat_sys: Option<i64>,
    pub pkg__id: Option<i64>,
    pub pkg_pkg: Option<String>,
    pub pkg_aiflag: Option<i64>,
    pub pkg_ast: Option<i64>,
    pub pkg_cat: Option<i64>,
    pub pkg_name: Option<String>,
    pub pkg_note: Option<String>,
    pub pkg_time: Option<i64>,
    pub pkg_type: Option<i64>,
    pub pkg_ucat: Option<i64>,
    pub pkg_ver: Option<String>,
}

use crate::extract::{properties::ExtractedInfo, ExtractInfo};
impl ExtractInfo for AppUsageEntry {
    fn extract_info(&self) -> Option<ExtractedInfo> {
        use crate::extract::properties::*;
        let x = &self;
        if x.act_type == crate::import::app_usage_sqlite::UseType::UseApp {
            let pkg_name = x.pkg_name.as_deref().unwrap_or("").to_string();
            Some(ExtractedInfo::UseDevice {
                general: GeneralSoftware {
                    hostname: x.device_name.clone(),
                    device_type: SoftwareDeviceType::Smartphone,
                    device_os: "Android".to_string(),
                    title: pkg_name.clone(),
                    identifier: Identifier(format!(
                        "android:{}",
                        x.pkg_pkg.as_deref().unwrap_or("??").to_string()
                    )),
                    unique_name: pkg_name.clone(),
                },
                specific: SpecificSoftware::Unknown,
            })
        } else {
            None
        }
    }
}

impl Importable for AppUsageImportArgs {
    fn import(&self) -> anyhow::Result<Vec<NewDbEvent>> {
        let conf = self;
        if !TIBU_FNAME.is_match(&conf.filename) {
            anyhow::bail!("Not a tibu file!");
        }
        let mut bytes = [0u8; 4096];
        std::fs::File::open(&conf.filename)?.read(&mut bytes)?;
        let cap = TIBU_ENCRYPTED.captures(&bytes);
        if let Some(c) = cap {
            // decrypt
            // let password = std::env::var("TIBU_PW").expect("Set env var TIBU_PW");
            anyhow::bail!("Encrypted!");
        } else {
            // anyhow::bail!("Not encrypted!");
            let mut a = tar::Archive::new(bzip2::read::BzDecoder::new(File::open(&conf.filename)?));
            let tmp = tempfile::TempDir::new()?;
            for e in a.entries()? {
                let mut e = e?;
                let path = e.header().path().unwrap().to_str().unwrap().to_string();
                if path.starts_with("data/data/com.a0soft.gphone.uninstaller/./databases/data.db") {
                    let pathb = PathBuf::from(path);
                    let mut out = tmp.path().to_path_buf();
                    out.push(pathb.file_name().unwrap());
                    println!("extracting to filename: {}", out.to_string_lossy());
                    let mut outf = File::create(&out)?;
                    std::io::copy(&mut e, &mut outf)?;
                }
            }
            let db_fname = {
                let mut out = tmp.path().to_path_buf();
                out.push("data.db");
                out
            };
            let conn = rusqlite::Connection::open(db_fname)?;
            print!("querying (each dot=10k events) ");
            let mut query = conn.prepare(
                "
            select 
            act._id as 'act__id',
            act.dur as 'act_dur',
            act.pid as 'act_pid',
            act.time as 'act_time',
            act.type as 'act_type',
            cat._id as 'cat__id',
            cat.cid as 'cat_cid',
            cat.name as 'cat_name',
            cat.sys as 'cat_sys',
            pkg._id as 'pkg__id',
            pkg.pkg as 'pkg_pkg',
            pkg.aiflag as 'pkg_aiflag',
            pkg.ast as 'pkg_ast',
            pkg.cat as 'pkg_cat',
            pkg.name as 'pkg_name',
            pkg.note as 'pkg_note',
            pkg.time as 'pkg_time',
            pkg.type as 'pkg_type',
            pkg.ucat as 'pkg_ucat',
            pkg.ver as 'pkg_ver'

            from act
            -- left join usage on usage.pid = act.pid
            left join pkg on act.pid = pkg._id
            left join cat on pkg.cat = cat.cid
            order by dur desc
        ",
            )?;
            let mut i = 0;
            let z = query.query_map(params![], |row| {
                if i % 10000 == 0 {
                    print!(".");
                    std::io::stdout().flush().ok();
                }
                i += 1;
                Ok(AppUsageEntry {
                    device_type: conf.device_type.clone(),
                    device_name: conf.device_name.clone(),
                    act_dur: row.get("act_dur")?,
                    act_pid: row.get("act_pid")?,
                    act_time: row.get("act_time")?,
                    act_type: row.get("act_type")?,
                    act_type_raw: row.get("act_type")?,
                    cat__id: row.get("cat__id")?,
                    cat_cid: row.get("cat_cid")?,
                    cat_name: row.get("cat_name")?,
                    cat_sys: row.get("cat_sys")?,
                    pkg__id: row.get("pkg__id")?,
                    pkg_pkg: row.get("pkg_pkg")?,
                    pkg_aiflag: row.get("pkg_aiflag")?,
                    pkg_ast: row.get("pkg_ast")?,
                    pkg_cat: row.get("pkg_cat")?,
                    pkg_name: row.get("pkg_name")?,
                    pkg_note: row.get("pkg_note")?,
                    pkg_time: row.get("pkg_time")?,
                    pkg_type: row.get("pkg_type")?,
                    pkg_ucat: row.get("pkg_ucat")?,
                    pkg_ver: row.get("pkg_ver")?,
                })
            })?;
            let mut outs = Vec::new();
            let sampler_sequence_id = uuid::Uuid::new_v4().to_hyphenated().to_string();
            for n in z {
                let n = n?;
                let timestamp = util::unix_epoch_millis_to_date(n.act_time);
                let duration = (n.act_dur as f64) / 1000.0;
                let id = format!("app_usage.{}_{}_{}", n.act_time, n.act_type_raw, n.act_pid);
                outs.push(
                    CreateNewDbEvent {
                        timestamp,
                        sampler: Sampler::Explicit { duration },
                        sampler_sequence_id: sampler_sequence_id.clone(),
                        // assume each app can only do one event of specific type in one ms
                        // needed becouse the _id column in the db is not declared AUTOINCREMENT so may be reused
                        id,
                        data: EventData::app_usage_v1(n),
                    }
                    .try_into()?,
                );
            }
            println!("");
            println!("got {} acts", outs.len());
            Ok(outs)
        }
    }
}
