use crate::prelude::*;
use diesel::prelude::*;

pub struct YieldEventsFromTrbttDatabase<'a> {
    pub db: &'a SqliteConnection,
    pub chunk_size: i64,
    pub last_fetched: Timestamptz,
    pub ascending: bool,
}
impl<'a> Iterator for YieldEventsFromTrbttDatabase<'a> {
    type Item = Vec<DbEvent>;
    fn next(&mut self) -> Option<Self::Item> {
        use crate::db::schema::events::dsl::*;
        let mut query = events.into_boxed();
        if self.ascending {
            query = query
                .filter(timestamp.gt(&self.last_fetched))
                .order(timestamp.asc());
        } else {
            query = query
                .filter(timestamp.lt(&self.last_fetched))
                .order(timestamp.desc());
        }
        let result: Vec<DbEvent> = query
            .limit(self.chunk_size)
            .load::<DbEvent>(self.db)
            .expect("db loading error not handled");
        log::debug!(
            "iterator fetching events ascending={}, start={:?}, limit={}, found={}",
            self.ascending,
            self.last_fetched,
            self.chunk_size,
            result.len()
        );
        if !result.is_empty() {
            self.last_fetched = result[result.len() - 1].timestamp.clone();
            Some(result)
        } else {
            // done, no more elements
            None
        }
    }
}
