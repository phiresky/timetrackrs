use chrono::Local;
use chrono::DateTime;
use super::schema::*;

#[derive(Queryable)]
pub struct Activity {
    pub id: i64,
    pub created: DateTime<Local>,
    pub data: String
}


#[derive(AsExpression, FromSqlRow, PartialEq, Debug, Clone)]
#[sql_type = "Text"]
pub struct Timestamptz {
    inner: DateTime<Local>
}
impl Timestamptz {
    pub fn now() -> Timestamptz {
        Timestamptz {
            inner: chrono::Local::now()
        }
    }
}

use diesel::sql_types::Text;
use diesel::sqlite::Sqlite;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, Output, ToSql};
use std::io::Write;

impl ToSql<Text, Sqlite> for Timestamptz {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Sqlite>) -> serialize::Result {
        let s = self.inner.to_rfc3339();
        <String as ToSql<Text, Sqlite>>::to_sql(&s, out)
    }
}

impl FromSql<Text, Sqlite> for Timestamptz {
    fn from_sql(bytes: Option<&<Sqlite as diesel::backend::Backend>::RawValue>) -> deserialize::Result<Self> {
        let s = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        return Ok(Timestamptz {
            inner: DateTime::<chrono::FixedOffset>::parse_from_rfc3339(&s)?.with_timezone(&chrono::Local),
        })
    }
}


#[derive(Insertable)]
#[table_name = "activity"]
pub struct NewActivity {
    pub created: Timestamptz,
    pub data: String
}