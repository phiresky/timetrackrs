#[macro_use]
extern crate diesel;

use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use track_pc_usage_rs as trbtt;
use trbtt::db::models::Timestamptz;
use trbtt::db::schema::events;

#[derive(Identifiable, Debug, Queryable, AsChangeset)]
#[table_name = "events"]
struct RefreshDate {
    id: String,
    timestamp: Timestamptz,
}

fn main() -> anyhow::Result<()> {
    /*let s = sysinfo::System::new();
    let process = s.get_process(953238);
    println!("{:?}", process);*/

    let db = trbtt::database::connect()?;

    use trbtt::db::schema::events::dsl::*;
    let mdata = events.select((id, timestamp)).load::<RefreshDate>(&db)?;
    // println!("{:?}", mdata);
    for x in mdata {
        if x.id.len() < 8 {
            let new_id = trbtt::util::random_uuid();
            let upd = diesel::update(events)
                .filter(id.eq(&x.id))
                .set(id.eq(new_id));
            let debug = diesel::debug_query::<diesel::sqlite::Sqlite, _>(&upd);
            println!("debug={}", debug);
            upd.execute(&db)?;
        }
    }
    Ok(())
}
