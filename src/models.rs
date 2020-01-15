use super::schema::*;
use chrono::DateTime;
use chrono::Utc;
use serde::Serialize;

#[derive(Queryable, Serialize)]
pub struct Activity {
    pub id: i32,
    pub timestamp: Timestamptz,
    pub data_type: String,
    pub data_type_version: i32,
    pub sampler: String,
    pub data: String,
}

#[derive(AsExpression, FromSqlRow, PartialEq, Debug, Clone, Serialize)]
#[sql_type = "Text"]
pub struct Timestamptz {
    inner: DateTime<Utc>,
}
impl Timestamptz {
    pub fn now() -> Timestamptz {
        Timestamptz {
            inner: chrono::Utc::now(),
        }
    }
    pub fn new(d: DateTime<Utc>) -> Timestamptz {
        Timestamptz { inner: d }
    }
}

use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;
use diesel::sqlite::Sqlite;
use std::io::Write;

impl ToSql<Text, Sqlite> for Timestamptz {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Sqlite>) -> serialize::Result {
        let s = self.inner.to_rfc3339();
        <String as ToSql<Text, Sqlite>>::to_sql(&s, out)
    }
}

impl FromSql<Text, Sqlite> for Timestamptz {
    fn from_sql(
        bytes: Option<&<Sqlite as diesel::backend::Backend>::RawValue>,
    ) -> deserialize::Result<Self> {
        let s = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        return Ok(Timestamptz {
            inner: DateTime::<chrono::FixedOffset>::parse_from_rfc3339(&s)?.with_timezone(&Utc),
        });
    }
}

#[derive(Insertable)]
#[table_name = "activity"]
pub struct NewActivity {
    pub timestamp: Timestamptz,
    pub data_type: String,
    pub data_type_version: i32,
    pub sampler: String,
    pub data: String,
}
