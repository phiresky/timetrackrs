use std::time::Instant;

use futures::Stream;
use sqlx::SqlitePool;

use crate::prelude::*;
/*
pub struct YieldEventsFromTrbttDatabase<'a> {
    pub db: SqlitePool,
    pub chunk_size: i64,
    pub last_fetched: Timestamptz,
    pub ascending: bool,
}
impl<'a> Stream for YieldEventsFromTrbttDatabase<'a> {
    type Item = Vec<DbEvent>;
    fn poll_next(&mut self) -> Option<Self::Item> {
        let result: Vec<DbEvent> = if self.ascending {
                let result: Vec<DbEvent> = sqlx::query_as!(
                    DbEvent,
                    "select * from events
                     where timestamp_unix_ms > ? order by timestamp_unix_ms asc limit ?",
                     &self.last_fetched, self.chunk_size
                ).fetch_all().await;
        } else {
            let result: Vec<DbEvent> = sqlx::query_as!(
                DbEvent,
                "select * from events
                 where timestamp_unix_ms < ? order by timestamp_unix_ms desc limit ?",
                 &self.last_fetched, self.chunk_size
            ).fetch_all().await;
        }
        let now = Instant::now();

        .fetch_all();
        let result: Vec<DbEvent> = query
            .limit(self.chunk_size)
            .load::<DbEvent>(self.db)
            .expect("db loading error not handled");
        log::debug!("fetching from db took {:?}s", now.elapsed());
        log::debug!(
            "iterator fetching events ascending={}, start={:?}, limit={}, found={}",
            self.ascending,
            self.last_fetched,
            self.chunk_size,
            result.len()
        );
        if !result.is_empty() {
            self.last_fetched = result[result.len() - 1].timestamp_unix_ms.clone();
            Some(result)
        } else {
            // done, no more elements
            None
        }
    }
}
*/
