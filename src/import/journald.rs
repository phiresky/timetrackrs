#![allow(non_snake_case)]

use crate::prelude::*;

use structopt::StructOpt;

#[derive(StructOpt)]
pub struct JournaldImportArgs {}

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct JournaldEntry {
    os_info: util::OsInfo,
    event: JournaldEvent,
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
enum JournaldEvent {
    Powerup,
    Shutdown,
    LogEntry(J),
}

impl ExtractInfo for JournaldEntry {
    fn extract_info(&self) -> Option<Tags> {
        let mut tags = Tags::new();
        tags.add("todo", "journald info");
        Some(tags)
        /*
        let mut general = self.os_info.to_partial_general_software();
        general.identifier = Identifier("exe:/usr/bin/journalctl".to_string());
        general.title = "journalctl".to_string();
        general.unique_name = "systemd".to_string();
        general.opened_filepath = None;
        Some(ExtractedInfo::InteractWithDevice {
            general,
            specific: SpecificSoftware::DeviceStateChange {
                change: match self.event {
                    JournaldEvent::Powerup => DeviceStateChange::PowerOn,
                    JournaldEvent::Shutdown => DeviceStateChange::PowerOff,
                    _ => return None,
                },
            },
        })*/
    }
}
lazy_static! {
    // example:
    // -74 daee0006297641feb18738955c7c125e Fri 2019-06-21 18:14:04 UTC—Fri 2019-06-21 20:29:58 UTC
    // -73 7754c38d30434a99b25640c1b32c1af3 Sat 2019-06-22 15:48:03 UTC—Thu 2019-06-27 23:13:27 UTC
    static ref JOURNALD_LIST_BOOTS: regex::Regex = regex::Regex::new(
        r#"(?x)
        ^ # start line
        \s*(?P<relative_boot_number>-?\d+)\  # relative boot number
        (?P<boot_id>[0-9a-f]+)\ 
        ...\ # week day
        (?P<start>\d\d\d\d-\d\d-\d\d\ \d\d:\d\d:\d\d)\ UTC
        —...\ (?P<end>\d\d\d\d-\d\d-\d\d\ \d\d:\d\d:\d\d)\ UTC
        $ # end line
        "#
    )
    .unwrap();
}
#[async_trait]
impl Importable for JournaldImportArgs {
    async fn import(&self) -> ImportResult {
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
                .with_context(|| format!("could no match output '{line}'"))?;
            let relative_boot_number = cap.name("relative_boot_number").unwrap().as_str();
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
                CreateNewDbEvent {
                    id: format!("{boot_id}.powerup"),
                    timestamp: start,
                    data: EventData::journald_v1(JournaldEntry {
                        os_info: os_info.clone(),
                        event: JournaldEvent::Powerup,
                    }),
                    duration_ms: 0,
                }
                .try_into()?,
            );
            // current boot has not shut down, so don't add shutdown event
            if relative_boot_number != "0" {
                outs.push(
                    CreateNewDbEvent {
                        id: format!("{boot_id}.shutdown"),
                        timestamp: end,
                        data: EventData::journald_v1(JournaldEntry {
                            os_info: os_info.clone(),
                            event: JournaldEvent::Shutdown,
                        }),
                        duration_ms: 0,
                    }
                    .try_into()?,
                );
            }
        }

        Ok(Box::pin(futures::stream::once(futures::future::ok(outs))))

        // journalctl -t systemd-sleep --output=json --all
    }
}
