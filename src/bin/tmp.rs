#[macro_use]
extern crate diesel;

use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use sysinfo::SystemExt;
use track_pc_usage_rs as trbtt;
use trbtt::models::Timestamptz;
use trbtt::schema::activity;

#[derive(Identifiable, Debug, Queryable, AsChangeset)]
#[table_name = "activity"]
struct RefreshDate {
    id: String,
    timestamp: Timestamptz,
}

fn main() -> anyhow::Result<()> {
    /*let s = sysinfo::System::new();
    let process = s.get_process(953238);
    println!("{:?}", process);*/

    let db = trbtt::database::connect()?;

    use trbtt::schema::activity::dsl::*;
    let mdata = activity.select((id, timestamp)).load::<RefreshDate>(&db)?;
    // println!("{:?}", mdata);
    for x in mdata {
        if (x.id.len() < 8) {
            let new_id = trbtt::util::random_uuid();
            let upd = diesel::update(activity)
                .filter(id.eq(&x.id))
                .set(id.eq(new_id));
            let debug = diesel::debug_query::<diesel::sqlite::Sqlite, _>(&upd);
            println!("debug={}", debug);
            upd.execute(&db)?;
        }
    }
    Ok(())
}
