#![allow(non_snake_case)]

use crate::prelude::*;

use structopt::StructOpt;

#[derive(StructOpt)]
pub struct JournaldImportArgs {}

#[derive(Debug, Serialize, Deserialize, TypeScriptify)]
pub struct JournaldEntry {
    os_info: util::OsInfo,
    event: JournaldEvent,
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify)]
enum JournaldEvent {
    Powerup,
    Shutdown,
    LogEntry(J),
}

impl ExtractInfo for JournaldEntry {
    fn extract_info(&self) -> Option<ExtractedInfo> {
        None
    }
}
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
        let os_info = util::get_os_info();

        let child = Command::new("journalctl")
            .arg("--list-boots")
            .arg("--utc")
            .stdout(Stdio::piped())
            .spawn()?;

        let output = BufReader::new(child.stdout.unwrap());
        let mut outs = vec![];
        for line in output.lines() {
            let line = line?;
            let cap = JOURNALD_LIST_BOOTS
                .captures(&line)
                .ok_or(anyhow::anyhow!("could no match output '{}'", line))?;
            let boot_id = cap.name("boot_id").unwrap().as_str();
            let start = cap.name("start").unwrap().as_str();
            let end = cap.name("end").unwrap().as_str();
            let start = DateTime::<Utc>::from_utc(
                NaiveDateTime::parse_from_str(start, "%Y-%m-%d %H:%M:%S")?,
                Utc,
            );
            let end = DateTime::<Utc>::from_utc(
                NaiveDateTime::parse_from_str(end, "%Y-%m-%d %H:%M:%S")?,
                Utc,
            );
            outs.push(
                CreateNewActivity {
                    id: format!("{}.powerup", boot_id),
                    timestamp: start,
                    data: CapturedData::journald(JournaldEntry {
                        os_info: os_info.clone(),
                        event: JournaldEvent::Powerup,
                    }),
                    sampler: Sampler::Explicit { duration: 0.0 },
                    sampler_sequence_id: "".to_string(),
                }
                .try_into()?,
            );
            outs.push(
                CreateNewActivity {
                    id: format!("{}.shutdown", boot_id),
                    timestamp: end,
                    data: CapturedData::journald(JournaldEntry {
                        os_info: os_info.clone(),
                        event: JournaldEvent::Shutdown,
                    }),
                    sampler: Sampler::Explicit { duration: 0.0 },
                    sampler_sequence_id: "".to_string(),
                }
                .try_into()?,
            );
        }
        let s = Sampler::Explicit { duration: 0.0 };

        Ok(outs)

        // journalctl -t systemd-sleep --output=json --all

        // journalctl --list-boots
    }
}
