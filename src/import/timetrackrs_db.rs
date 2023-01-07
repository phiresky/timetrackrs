use std::str::FromStr;

use crate::prelude::*;
use futures::StreamExt;
use sqlx::{sqlite::SqliteConnectOptions, ConnectOptions};

#[derive(StructOpt)]
pub struct TimetrackrsImportArgs {
    /// path to import raw_events.sqlite3
    filename: String,
    /// the previously seen max sequence. skips importing older events defaults to 0
    last_id: Option<i64>,
}
#[async_trait]
impl Importable for TimetrackrsImportArgs {
    async fn import(&self) -> ImportResult {
        let mut db = SqliteConnectOptions::from_str(&self.filename)?
            .connect()
            .await?;
        // double open because of https://github.com/launchbadge/sqlx/issues/832
        sqlx::query(&format!(
            "attach database '{}' as raw_events;",
            self.filename
        ))
        .execute(&mut db)
        .await?;
        let x = sqlx::query_scalar!("select count(*) as c from raw_events.events")
            .fetch_one(&mut db)
            .await?;
        println!("have {x}");

        let db = Box::leak(Box::new(db)); // shh bby is ok
        let last_id = Box::leak(self.last_id.unwrap_or(0).into());
        let is_new_db: i32 = sqlx::query_scalar!(
            "select count(*) from pragma_table_info('events') where name = 'timestamp_unix_ms'"
        )
        .fetch_one(&mut *db)
        .await?;
        let raws = {
            if is_new_db == 1 {
                sqlx::query_as!(
                    NewDbEvent,
                    r#"select
                id, timestamp_unix_ms as "timestamp_unix_ms: _",
                data_type, duration_ms, data
            from raw_events.events where insertion_sequence > ?"#,
                    *last_id
                ).fetch(db)
            } else {
                println!("detected legacy db format");
                sqlx::query_as(r#"
                SELECT
                    id,
                    cast(round((julianday (timestamp) - 2440587.5) * 86400.0 * 1000) AS int) AS timestamp_unix_ms,
                    data_type,
                    cast(round(coalesce(json_extract (sampler, '$.avg_time'), json_extract (sampler, '$.duration')) * 1000) AS int) AS duration_ms,
                    data
                FROM
                    raw_events.events"#).fetch(db)
            }
        }
        .chunks(1000)
        .map(|e| {
            e.into_iter()
                .collect::<sqlx::Result<Vec<NewDbEvent>>>()
                .context("chunk")
        });
        Ok(Box::pin(raws))
    }
}
/* INSERT INTO raw_events.events
SELECT
    insertion_sequence,
    id,
    cast(round((julianday (timestamp) - 2440587.5) * 86400.0 * 1000) AS int) AS timestamp_unix_ms,
    data_type,
    cast(round(coalesce(json_extract (sampler, '$.avg_time'), json_extract (sampler, '$.duration')) * 1000) AS int) AS duration_ms,
    data
FROM
    events_backup_before_2021_01;

update events set timestamp_unix_ms = timestamp_unix_ms - timestamp_unix_ms%duration_ms where insertion_sequence <= 848066 and data_type = 'x11_v2'
 */
