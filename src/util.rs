use chrono::prelude::*;

pub fn unix_epoch_millis_to_date(timestamp: i64) -> DateTime<Utc> {
    let timestamp_s = timestamp / 1000;
    let timestamp_us = (timestamp % 1000) * 1_000_000;
    let naive_datetime = NaiveDateTime::from_timestamp(timestamp_s, timestamp_us as u32);
    DateTime::from_utc(naive_datetime, Utc)
}
