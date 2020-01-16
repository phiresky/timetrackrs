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
    id: i64,
    timestamp: Timestamptz,
}

fn main() -> anyhow::Result<()> {
    /*let s = sysinfo::System::new();
    let process = s.get_process(953238);
    println!("{:?}", process);*/

    let db = trbtt::database::connect()?;

    use trbtt::schema::activity::dsl::*;
    let mdata = activity.select((id, timestamp)).load::<RefreshDate>(&db)?;
    println!("{:?}", mdata);
    for x in mdata {
        let upd = diesel::update(activity).filter(id.eq(x.id)).set(&x);
        let debug = diesel::debug_query(&upd);
        println!("debug={}", debug);
        upd.execute(&db)?;
    }
    Ok(())
}
