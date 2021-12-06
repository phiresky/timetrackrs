use crate::prelude::*;
use anyhow::Context;
use serde::{de::Visitor, Deserializer, Serializer};
use sqlx::{
    sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef},
    Decode, Sqlite,
};
use std::fmt;

#[derive(Serialize, TypeScriptify)]
pub struct DbEvent {
    pub insertion_sequence: i64,
    pub id: String,
    pub timestamp_unix_ms: Timestamptz,
    pub data_type: String,
    pub duration_ms: i64,
    pub data: String,
}

impl DbEvent {
    pub fn deserialize_data(&self) -> anyhow::Result<EventData> {
        deserialize_captured((&self.data_type, &self.data))
            .with_context(|| format!("deserialization of event {}", self.id))
    }
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Eq, Hash, Copy)]
pub struct Timestamptz(pub DateTime<Utc>);

impl sqlx::Type<Sqlite> for Timestamptz {
    fn type_info() -> SqliteTypeInfo {
        // use i32 so the sqlite type is INTEGER and not INT8 or BIGINT
        <i32 as sqlx::Type<Sqlite>>::type_info()
    }
    /*fn compatible(ty: &SqliteTypeInfo) -> bool {
        log::info!("got type info {}", ty);
        return *ty == <i64 as sqlx::Type<Sqlite>>::type_info();
    }*/
}
impl sqlx::Encode<'_, Sqlite> for Timestamptz {
    fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue<'_>>) -> sqlx::encode::IsNull {
        sqlx::Encode::<Sqlite>::encode(self.0.timestamp_millis(), buf)
    }
}
impl Decode<'_, Sqlite> for Timestamptz {
    fn decode(
        value: SqliteValueRef,
    ) -> Result<Timestamptz, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <i64 as Decode<Sqlite>>::decode(value).map_err(|e| {
            log::info!("ERROR EEE {:?}", e);
            e
        })?;
        Ok(Timestamptz(util::unix_epoch_millis_to_date(value)))
    }
}
/*
impl FromSql<BigInt, Sqlite> for Timestamptz {
    fn from_sql(
        bytes: Option<&<Sqlite as diesel::backend::Backend>::RawValue>,
    ) -> deserialize::Result<Self> {
        let i = <i64 as FromSql<BigInt, Sqlite>>::from_sql(bytes)?;
        Ok(Timestamptz(util::unix_epoch_millis_to_date(i)))
    }
}
impl ToSql<BigInt, Sqlite> for Timestamptz {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Sqlite>) -> serialize::Result {
        let s = self.0.timestamp_millis();
        <i64 as ToSql<BigInt, Sqlite>>::to_sql(&s, out)
    }
}*/
impl From<&Timestamptz> for Timestamptz {
    fn from(t: &Timestamptz) -> Self {
        Self(t.0)
    }
}

impl Serialize for Timestamptz {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i64(self.0.timestamp_millis())
    }
}

struct TimestamptzVisitor;

impl<'de> Visitor<'de> for TimestamptzVisitor {
    type Value = Timestamptz;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a unix timestamp")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Timestamptz(util::unix_epoch_millis_to_date(value)))
    }
    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(value as i64)
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Timestamptz(
            util::iso_string_to_datetime(value).map_err(serde::de::Error::custom)?,
        ))
    }
}
impl<'de> Deserialize<'de> for Timestamptz {
    fn deserialize<D>(deserializer: D) -> Result<Timestamptz, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(TimestamptzVisitor)
    }
}

#[derive(PartialEq, PartialOrd, Ord, Debug, Clone, Eq, Hash, Copy)]
pub struct TimeChunk(DateTime<Utc>);

pub const MAX_EVENT_LEN_SECS: i64 = 24 * 60 * 60;
pub const CHUNK_LEN_MINS: u32 = 5;
impl TimeChunk {
    pub fn containing(time: DateTime<Utc>) -> TimeChunk {
        let time = time
            .with_nanosecond(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_minute(time.minute() - time.minute() % CHUNK_LEN_MINS)
            .unwrap();
        TimeChunk(time)
    }
    pub fn at(time: DateTime<Utc>) -> anyhow::Result<TimeChunk> {
        if time.minute() % CHUNK_LEN_MINS != 0 {
            anyhow::bail!("not {}-minute chunk", CHUNK_LEN_MINS);
        }
        if time.second() != 0 {
            anyhow::bail!("second != 0");
        }
        if time.nanosecond() != 0 {
            anyhow::bail!("nanosecond != 0")
        }
        Ok(TimeChunk(time))
    }
    fn to_string(&self) -> String {
        format!("{}", self.0.format("%F-%R"))
    }
    fn from_string(s: &str) -> anyhow::Result<TimeChunk> {
        let time = DateTime::from_utc(
            NaiveDateTime::parse_from_str(s, "%F-%R").context("timechunk parser")?,
            Utc,
        );

        TimeChunk::at(time)
    }
    pub fn start(&self) -> DateTime<Utc> {
        self.0
    }
    pub fn end_exclusive(&self) -> DateTime<Utc> {
        self.0 + chrono::Duration::minutes(CHUNK_LEN_MINS as i64)
    }
}

impl sqlx::Type<Sqlite> for TimeChunk {
    fn type_info() -> SqliteTypeInfo {
        <String as sqlx::Type<Sqlite>>::type_info()
    }
}
impl sqlx::Encode<'_, Sqlite> for TimeChunk {
    fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue<'_>>) -> sqlx::encode::IsNull {
        sqlx::Encode::<Sqlite>::encode(self.to_string(), buf)
    }
}
impl Decode<'_, Sqlite> for TimeChunk {
    fn decode(
        value: SqliteValueRef,
    ) -> Result<TimeChunk, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <String as Decode<Sqlite>>::decode(value)?;
        Ok(TimeChunk::from_string(&value)?)
    }
}

struct TimeChunkVisitor;

impl<'de> Visitor<'de> for TimeChunkVisitor {
    type Value = TimeChunk;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a time chunk value")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        TimeChunk::from_string(value).map_err(serde::de::Error::custom)
    }
}
impl<'de> Deserialize<'de> for TimeChunk {
    fn deserialize<D>(deserializer: D) -> Result<TimeChunk, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(TimeChunkVisitor)
    }
}

impl Serialize for TimeChunk {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

/*
impl FromSql<Text, Sqlite> for DateUtc {
    fn from_sql(
        bytes: Option<&<Sqlite as diesel::backend::Backend>::RawValue>,
    ) -> deserialize::Result<Self> {
        let s = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        Ok(DateUtc(util::iso_string_to_date(&s)?))
    }
}
impl ToSql<Text, Sqlite> for DateUtc {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Sqlite>) -> serialize::Result {
        let s = format!("{}", self.0.format("%F"));
        <String as ToSql<Text, Sqlite>>::to_sql(&s, out)
    }
}*/

#[derive(sqlx::FromRow)]
pub struct NewDbEvent {
    pub id: String,
    pub timestamp_unix_ms: Timestamptz,
    pub data_type: String,
    pub duration_ms: i64,
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone, sqlx::Type)]
pub struct TagRuleGroup {
    pub global_id: String,
    pub data: sqlx::types::Json<TagRuleGroupData>,
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
#[serde(tag = "version")]
pub enum TagRuleGroupData {
    V1 { data: TagRuleGroupV1 },
}

impl TagRuleGroupData {
    pub fn into_iter_active_rules(self) -> impl Iterator<Item = TagRule> {
        match self {
            TagRuleGroupData::V1 { data } => {
                data.rules
                    .into_iter()
                    .filter_map(|r| if r.enabled { Some(r.rule) } else { None })
            }
        }
    }
    pub fn into_iter_all_rules(self) -> impl Iterator<Item = TagRule> {
        match self {
            TagRuleGroupData::V1 { data } => data.rules.into_iter().map(|r| r.rule),
        }
    }
}
/*
impl FromSql<Text, Sqlite> for TagRuleGroupData {
    fn from_sql(
        bytes: Option<&<Sqlite as diesel::backend::Backend>::RawValue>,
    ) -> deserialize::Result<Self> {
        let s = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        Ok(serde_json::from_str(&s)?)
    }
}
impl ToSql<Text, Sqlite> for TagRuleGroupData {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Sqlite>) -> serialize::Result {
        let s = serde_json::to_string(&self)?;
        <String as ToSql<Text, Sqlite>>::to_sql(&s, out)
    }
}*/
