

use crate::prelude::*;
use diesel::prelude::*;

#[derive(StructOpt)]
pub struct TrbttImportArgs {
    filename: String,
    after: Option<String>,
    limit: Option<i64>,
}
struct YieldAllEventsFromTrbttDatabase {
    db: SqliteConnection,
    last_id: i64,
}
impl Iterator for YieldAllEventsFromTrbttDatabase {
    type Item = Vec<NewDbEvent>;
    fn next(&mut self) -> Option<Self::Item> {
        use crate::db::schema::events::dsl::*;
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
                        timestamp: e.timestamp,
                        data_type: e.data_type,
                        sampler: e.sampler,
                        sampler_sequence_id: e.sampler_sequence_id,
                    })
                    .collect(),
            )
        } else {
            // done
            None
        }
    }
}
impl Importable for TrbttImportArgs {
    fn import(&self) -> ImportResult {
        let db = crate::db::connect_file(&self.filename)?;

        Ok(Box::new(YieldAllEventsFromTrbttDatabase { db, last_id: 0 }))
    }
}
