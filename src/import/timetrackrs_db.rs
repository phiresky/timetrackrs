use std::str::FromStr;

use crate::prelude::*;
use futures::StreamExt;
use sqlx::{sqlite::SqliteConnectOptions, ConnectOptions, SqliteConnection, SqlitePool};

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
        println!("have {}", x);

        let db = Box::leak(Box::new(db)); // shh bby is ok
        let last_id = Box::leak(self.last_id.unwrap_or(0).into());
        let raws = sqlx::query_as!(
            NewDbEvent,
            r#"select
                id, timestamp_unix_ms as "timestamp_unix_ms: _",
                data_type, duration_ms, data
            from raw_events.events where insertion_sequence > ?"#,
            *last_id
        )
        .fetch(db)
        .chunks(1000)
        .map(|e| {
            e.into_iter()
                .collect::<sqlx::Result<Vec<NewDbEvent>>>()
                .context("chunk")
        });
        Ok(Box::pin(raws))
    }
}
