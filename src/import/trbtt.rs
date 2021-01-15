use crate::prelude::*;
use diesel::prelude::*;

#[derive(StructOpt)]
pub struct TrbttImportArgs {
    /// path to import events.sqlite3
    filename: String,
    /// the previously seen max sequence. skips importing older events defaults to 0
    last_id: Option<i64>,
}
struct YieldAllEventsFromTrbttDatabase {
    db: SqliteConnection,
    last_id: i64,
}
impl Iterator for YieldAllEventsFromTrbttDatabase {
    type Item = Vec<NewDbEvent>;
    fn next(&mut self) -> Option<Self::Item> {
        use crate::db::schema::raw_events::events::dsl::*;
        let result: Vec<DbEvent> = events
            .filter(insertion_sequence.gt(self.last_id))
            .order(insertion_sequence.asc())
            .limit(10000)
            .load::<DbEvent>(&self.db)
            .expect("db loading error not handled");
        if !result.is_empty() {
            self.last_id = result[result.len() - 1].insertion_sequence;
            Some(
                result
                    .into_iter()
                    .map(|e| NewDbEvent {
                        id: e.id,
                        data: e.data,
                        timestamp_unix_ms: e.timestamp_unix_ms,
                        data_type: e.data_type,
                        duration_ms: e.duration_ms,
                    })
                    .collect(),
            )
        } else {
            log::info!("final import sequence id: {}", self.last_id);
            // done
            None
        }
    }
}
impl Importable for TrbttImportArgs {
    fn import(&self) -> ImportResult {
        let db = crate::db::raw_events::connect_file(&self.filename)?;

        Ok(Box::new(YieldAllEventsFromTrbttDatabase {
            db,
            last_id: self.last_id.unwrap_or(0),
        }))
    }
}
