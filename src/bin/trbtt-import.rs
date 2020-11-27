use diesel::prelude::*;

use track_pc_usage_rs as trbtt;
use trbtt::prelude::*;

fn main() -> anyhow::Result<()> {
    util::init_logging();

    let opt = ImportArgs::from_args();
    let data = opt.import()?;
    log::info!("inserting...");
    use track_pc_usage_rs::db::schema::events;
    let db = track_pc_usage_rs::db::connect()?;

    let updated = diesel::insert_or_ignore_into(events::table).values(&data);

    let updated = updated
        .execute(&db)
        .context("inserting new events into db")?;

    log::info!(
        "successfully inserted {}/{} entries ({} already existed)",
        updated,
        data.len(),
        data.len() - updated
    );
    Ok(())
}
