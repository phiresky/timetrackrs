#[macro_use]
extern crate diesel;

use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use track_pc_usage_rs as trbtt;
use trbtt::db::models::Timestamptz;
use trbtt::db::schema::events;

fn main() -> anyhow::Result<()> {
    Ok(())
}
