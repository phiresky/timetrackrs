use crate::prelude::*;
use crate::schema::activity;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;
use diesel::sqlite::Sqlite;
use std::io::Write;
use uuid::Uuid;

#[derive(Queryable, Serialize, TypeScriptify)]
pub struct Activity {
    pub id: String,
    pub timestamp: Timestamptz,
    pub data_type: String,
    pub sampler: Sampler,
    pub sampler_sequence_id: String,
    pub data: String,
}

#[derive(AsExpression, FromSqlRow, PartialEq, Debug, Clone, Serialize)]
#[sql_type = "Text"]
#[serde(untagged)]
pub enum Timestamptz {
    N(DateTime<Utc>),
}
impl Timestamptz {
    pub fn new(d: DateTime<Utc>) -> Timestamptz {
        Timestamptz::N(d)
    }
}

impl FromSql<Text, Sqlite> for Timestamptz {
    fn from_sql(
        bytes: Option<&<Sqlite as diesel::backend::Backend>::RawValue>,
    ) -> deserialize::Result<Self> {
        let s = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        Ok(Timestamptz::new(util::iso_string_to_date(&s)?))
    }
}
impl ToSql<Text, Sqlite> for Timestamptz {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Sqlite>) -> serialize::Result {
        match &self {
            Timestamptz::N(d) => {
                let s = d.to_rfc3339();
                <String as ToSql<Text, Sqlite>>::to_sql(&s, out)
            }
        }
    }
}

impl FromSql<Text, Sqlite> for Sampler {
    fn from_sql(
        bytes: Option<&<Sqlite as diesel::backend::Backend>::RawValue>,
    ) -> deserialize::Result<Self> {
        let s = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        Ok(serde_json::from_str(&s)?)
    }
}
impl ToSql<Text, Sqlite> for Sampler {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Sqlite>) -> serialize::Result {
        let s = serde_json::to_string(&self)?;
        <String as ToSql<Text, Sqlite>>::to_sql(&s, out)
    }
}

#[derive(Insertable)]
#[table_name = "activity"]
pub struct NewActivity {
    pub id: String,
    pub timestamp: Timestamptz,
    pub data_type: String,
    pub sampler: Sampler,
    pub sampler_sequence_id: String,
    pub data: String,
}
