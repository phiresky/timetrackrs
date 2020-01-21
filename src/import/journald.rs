#![allow(non_snake_case)]

use crate::import::Importable;
use crate::models::NewActivity;
use crate::sampler::Sampler;
use crate::util::iso_string_to_date;
use crate::util::unix_epoch_millis_to_date;
use chrono::prelude::*;
use lazy_static::lazy_static;
use num_enum::TryFromPrimitive;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use structopt::StructOpt;
use typescript_definitions::TypeScriptify;

#[derive(StructOpt)]
pub struct JournaldImportArgs {}

/*pub struct JournaldBoot {

}*/

lazy_static! {
    // example:
    // -74 daee0006297641feb18738955c7c125e Fri 2019-06-21 18:14:04 UTC—Fri 2019-06-21 20:29:58 UTC
    // -73 7754c38d30434a99b25640c1b32c1af3 Sat 2019-06-22 15:48:03 UTC—Thu 2019-06-27 23:13:27 UTC
    static ref JOURNALD_LIST_BOOTS: regex::Regex = regex::Regex::new(
        r#"(?x)
        ^ # start line
        \s*-?\d+\  # relative boot number
        (?P<boot_id>[0-9a-f]+)\ 
        ...\ # week day
        (?P<start>\d\d\d\d-\d\d-\d\d\ \d\d:\d\d:\d\d)\ UTC
        —...\ (?P<end>\d\d\d\d-\d\d-\d\d\ \d\d:\d\d:\d\d)\ UTC
        $ # end line
        "#
    )
    .unwrap();
}
impl Importable for JournaldImportArgs {
    fn import(&self) -> anyhow::Result<Vec<NewActivity>> {
        use std::io::{BufRead, BufReader};
        use std::process::{Command, Stdio};

        let mut child = Command::new("journalctl")
            .arg("--list-boots")
            .arg("--utc")
            .stdout(Stdio::piped())
            .spawn()?;

        let output = BufReader::new(child.stdout.unwrap());
        // let v = vec![];
        for line in output.lines() {
            let line = line?;
            let cap = JOURNALD_LIST_BOOTS
                .captures(&line)
                .ok_or(anyhow::anyhow!("could no match output '{}'", line))?;
            let boot_id = cap.name("boot_id").unwrap().as_str();
            let start = cap.name("start").unwrap().as_str();
            let end = cap.name("end").unwrap().as_str();
            println!(
                "{} @ {}={:#?}",
                boot_id,
                start,
                DateTime::<Utc>::from_utc(
                    NaiveDateTime::parse_from_str(start, "%Y-%m-%d %H:%M:%S")?,
                    Utc
                )
            );
        }
        let s = Sampler::Explicit { duration: 0.0 };

        Ok(vec![])

        // journalctl -t systemd-sleep --output=json --all

        // journalctl --list-boots
    }
}
