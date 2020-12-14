use diesel::prelude::*;

use track_pc_usage_rs as trbtt;
use trbtt::prelude::*;

fn main() -> anyhow::Result<()> {
    util::init_logging();

    let opt = ImportArgs::from_args();
    let data = opt.import()?;
    log::info!("inserting...");
    use track_pc_usage_rs::db::schema::raw_events::events;
    let db = track_pc_usage_rs::db::raw_events::connect()?;

    let mut total_updated = 0;
    let mut total_seen = 0;
    let mut total_existed = 0;
    for data in data {
        let updated = diesel::insert_or_ignore_into(events::table).values(&data);

        let updated = updated
            .execute(&db)
            .context("inserting new events into db")?;
        total_updated += updated;
        total_seen += data.len();
        total_existed += data.len() - updated;
        log::info!(
            "successfully inserted {}/{} entries ({} already existed)",
            total_updated,
            total_seen,
            total_existed
        );
    }
    Ok(())
}
