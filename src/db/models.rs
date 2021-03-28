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

#[derive(PartialEq, PartialOrd, Debug, Clone, Eq, Hash)]
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
}
impl<'de> Deserialize<'de> for Timestamptz {
    fn deserialize<D>(deserializer: D) -> Result<Timestamptz, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_i64(TimestamptzVisitor)
    }
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Eq, Hash)]
pub struct DateUtc(pub Date<Utc>);

impl sqlx::Type<Sqlite> for DateUtc {
    fn type_info() -> SqliteTypeInfo {
        <String as sqlx::Type<Sqlite>>::type_info()
    }
}
impl sqlx::Encode<'_, Sqlite> for DateUtc {
    fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue<'_>>) -> sqlx::encode::IsNull {
        sqlx::Encode::<Sqlite>::encode(format!("{}", self.0.format("%F")), buf)
    }
}
impl Decode<'_, Sqlite> for DateUtc {
    fn decode(
        value: SqliteValueRef,
    ) -> Result<DateUtc, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <String as Decode<Sqlite>>::decode(value)?;
        Ok(DateUtc(util::iso_string_to_date(&value)?))
    }
}

struct DateUtcVisitor;

impl<'de> Visitor<'de> for DateUtcVisitor {
    type Value = DateUtc;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a date")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(DateUtc(
            util::iso_string_to_date(value).map_err(serde::de::Error::custom)?,
        ))
    }
}
impl<'de> Deserialize<'de> for DateUtc {
    fn deserialize<D>(deserializer: D) -> Result<DateUtc, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(DateUtcVisitor)
    }
}

impl Serialize for DateUtc {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("{}", self.0.format("%F")))
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

pub struct NewDbEvent {
    pub id: String,
    pub timestamp_unix_ms: Timestamptz,
    pub data_type: String,
    pub duration_ms: i64,
    pub data: String,
}

pub struct InExtractedTag {
    pub timestamp_unix_ms: Timestamptz,
    pub duration_ms: i64,
    pub event_id: i64,
    pub tag: i64,
    pub value: i64,
}
#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct OutExtractedTag {
    pub timestamp: Timestamptz,
    pub duration_ms: i64,
    pub tag: String,
    pub value: String,
    pub event_id: String,
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
