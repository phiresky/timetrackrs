use crate::prelude::*;
use std::{fmt::Display, iter::FromIterator, str::FromStr};

pub fn unix_epoch_millis_to_date(timestamp: i64) -> DateTime<Utc> {
    let timestamp_s = timestamp / 1000;
    let timestamp_us = (timestamp % 1000) * 1_000_000;
    let naive_datetime = NaiveDateTime::from_timestamp(timestamp_s, timestamp_us as u32);
    DateTime::from_utc(naive_datetime, Utc)
}

/*fn timestamp_to_iso_string(timestamp: i64) -> String {
    unix_epoch_millis_to_date(timestamp).to_rfc3339()
}*/

pub fn iso_string_to_datetime(s: &str) -> anyhow::Result<DateTime<Utc>> {
    // https://tc39.es/proposal-temporal/docs/iso-string-ext.html
    // allow time zone suffix, e.g. 2007-12-03T10:15:30+01:00[Europe/Paris]
    if s.ends_with(']') {
        let splitchar = s.rfind('[').context("Invalid date, broken TZ")?;
        let (s, _tz) = (&s[0..splitchar], &s[splitchar..]);
        //let tz = chrono_tz::Tz::from_str(tz)
        //    .map_err(|e| anyhow::anyhow!("could not parse tz: {e}"))?;

        return Ok(
            DateTime::<chrono::FixedOffset>::parse_from_rfc3339(s)?.with_timezone(&chrono::Utc)
        );
    }
    Ok(DateTime::<chrono::FixedOffset>::parse_from_rfc3339(s)
        .context("iso_string_to_datetime")?
        .with_timezone(&chrono::Utc))
}

pub fn iso_string_to_date(s: &str) -> anyhow::Result<Date<Utc>> {
    Ok(Date::from_utc(
        NaiveDate::parse_from_str(s, "%F").context("iso_string_to_date")?,
        Utc,
    ))
}

pub fn random_uuid() -> String {
    uuid::Uuid::new_v4().hyphenated().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeScriptify)]
pub struct OsInfo {
    // e.g. "Arch Linux" or "Windows"
    pub os_type: String,
    pub version: String,
    pub batteries: Option<i32>, // useful for determining pc vs laptop
    pub hostname: String,
    pub username: Option<String>,
    pub machine_id: Option<String>,
}
// TODO: remove defaults after rewrite
impl Default for OsInfo {
    fn default() -> OsInfo {
        OsInfo {
            os_type: "Arch Linux".to_string(),
            version: "rolling".to_string(),
            batteries: Some(0),
            hostname: "phirearch".to_string(),
            machine_id: None,
            username: Some("".to_string()),
        }
    }
}
impl OsInfo {
    pub fn to_partial_general_software(&self, tags: &mut Tags) {
        tags.add("use-device", "computer");
        tags.add("device-os-type".to_string(), &self.os_type);
        tags.add("device-os-version".to_string(), &self.version);
        tags.add("device-hostname".to_string(), &self.hostname);
        if let Some(m) = self.username.as_ref() {
            tags.add("device-username".to_string(), m)
        }
        if let Some(m) = self.machine_id.as_ref() {
            tags.add("device-machine-id".to_string(), m)
        }
        tags.add(
            "device-type".to_string(),
            format!(
                "{}",
                if self.batteries.unwrap_or(0) > 0 {
                    SoftwareDeviceType::Laptop
                } else {
                    SoftwareDeviceType::Desktop
                }
            ),
        );
    }
}

pub fn get_os_info() -> OsInfo {
    let os_info1 = os_info::get();
    let batteries = battery::Manager::new()
        .and_then(|e| e.batteries())
        .map(|e| e.count() as i32)
        .ok();
    let machine_id = std::fs::read_to_string("/etc/machine-id")
        .map(|e| e.trim().to_string())
        .ok();
    OsInfo {
        os_type: os_info1.os_type().to_string(),
        version: format!("{}", os_info1.version()),
        hostname: gethostname::gethostname().to_string_lossy().to_string(),
        machine_id,
        batteries,
        username: Some(whoami::username()),
    }
}

use tracing_subscriber::layer::SubscriberExt;

pub fn init_logging() -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "timetrackrs=info");
    }
    env_logger::init();
    let path = crate::db::get_database_dir_location().join("logs");
    std::fs::create_dir_all(&path).unwrap();
    let file_appender = tracing_appender::rolling::daily(path, "timetrackrs.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    // env_logger::init();
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
            .finish()
            .with(tracing_subscriber::fmt::Layer::default().with_writer(file_writer)),
    )?;
    log::debug!("env logger inited");
    Ok(guard)
}

use serde::de::{self, Deserializer, Visitor};
// https://github.com/serde-rs/serde/issues/581
pub fn comma_separated<'de, V, T, D>(deserializer: D) -> Result<V, D::Error>
where
    V: std::iter::FromIterator<T>,
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    use std::marker::PhantomData as Phantom;
    struct CommaSeparated<V, T>(Phantom<V>, Phantom<T>);

    impl<'de, V, T> Visitor<'de> for CommaSeparated<V, T>
    where
        V: std::iter::FromIterator<T>,
        T: FromStr,
        T::Err: Display,
    {
        type Value = V;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string containing comma-separated elements")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let iter = s.split(',').map(FromStr::from_str);
            Result::from_iter(iter).map_err(de::Error::custom)
        }
    }

    let visitor = CommaSeparated(Phantom, Phantom);
    deserializer.deserialize_str(visitor)
}
