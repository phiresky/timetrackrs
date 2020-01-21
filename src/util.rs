use chrono::prelude::*;

pub fn unix_epoch_millis_to_date(timestamp: i64) -> DateTime<Utc> {
    let timestamp_s = timestamp / 1000;
    let timestamp_us = (timestamp % 1000) * 1_000_000;
    let naive_datetime = NaiveDateTime::from_timestamp(timestamp_s, timestamp_us as u32);
    DateTime::from_utc(naive_datetime, Utc)
}

pub fn iso_string_to_date(s: &str) -> anyhow::Result<DateTime<Utc>> {
    Ok(DateTime::<chrono::FixedOffset>::parse_from_rfc3339(s)?.with_timezone(&chrono::Utc))
}

pub fn random_uuid() -> String {
    uuid::Uuid::new_v4().to_hyphenated().to_string()
}
