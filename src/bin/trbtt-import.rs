use futures::stream::StreamExt;
use timetrackrs::{prelude::*};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    util::init_logging();

    let opt = ImportArgs::from_args();
    let mut data = opt.import()?;
    log::info!("inserting...");
    let db = init_db_pool().await?;

    let mut total_updated: u64 = 0;
    let mut total_seen: u64 = 0;
    let mut total_existed: u64 = 0;
    while let Some(chunk) = data.next().await {
        let chunk = chunk?;
        let len = chunk.len() as u64;
        let updated = db.insert_events(chunk).await.context("inserting events")?;
        total_updated += updated;
        total_seen += len;
        total_existed += len - updated;
        log::info!(
            "successfully inserted {}/{} entries ({} already existed)",
            total_updated,
            total_seen,
            total_existed
        );
    }
    Ok(())
}
