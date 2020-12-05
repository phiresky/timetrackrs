use crate::util::iso_string_to_date;

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
    last_id: i64
}
impl Iterator for YieldAllEventsFromTrbttDatabase {
    type Item=Vec<NewDbEvent>;
    fn next(&mut self) -> Option<Item> {

    }
}
impl Importable for TrbttImportArgs {
    fn import(&self) -> ImportResult {
        let db = crate::db::connect_file(&self.filename)?;

        let mdata = {
            use crate::db::schema::events::dsl::*;
            let mut query = events.into_boxed();
            if let Some(after) = &self.after {
                let after = iso_string_to_date(&after)?;
                query = query
                    .filter(timestamp.gt(Timestamptz::new(after)))
                    .order(timestamp.asc());
            }
            /* if let Some(before) = before {
                let before = iso_string_to_date(&before)?;
                query = query
                    .filter(timestamp.lt(Timestamptz::new(before)))
                    .order(timestamp.desc());
            }*/
            let limit = self.limit.unwrap_or(1000000);
            query.limit(limit as i64).load::<DbEvent>(&db)?
        };
        Ok(Box::new(mdata
            .into_iter()
            .map(|e| NewDbEvent {
                id: e.id,
                data: e.data,
                timestamp: e.timestamp,
                data_type: e.data_type,
                sampler: e.sampler,
                sampler_sequence_id: e.sampler_sequence_id,
            })
        ))
    }
}
