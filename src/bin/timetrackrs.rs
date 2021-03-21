use futures::TryStreamExt;
use futures::{
    future::{BoxFuture, OptionFuture},
    never::Never,
    stream::FuturesUnordered,
    TryFuture,
};
use track_pc_usage_rs as trbtt;

use trbtt::prelude::*;
use trbtt::util::init_logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();
    let db = init_db_pool().await?;
    let config = trbtt::config::default_config();

    let features: FuturesUnordered<BoxFuture<anyhow::Result<Never>>> = FuturesUnordered::new();

    for c in config.capturers {
        features.push(Box::pin(capture_loop(db.clone(), c)));
    }
    if let Some(server) = config.server {
        features.push(Box::pin(trbtt::server::server::run_server(
            db.clone(),
            server,
        )));
    }

    let results: Vec<_> = features
        .try_collect()
        .await
        .context("Some feature failed")?;
    // features.await;
    /*let db = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite://foo.sqlite3")
        .await?;

    let q = sqlx::query!("select count(*) as coint from foo.events")
        .fetch_one(&db)
        .await?;
    println!("qq = {:?}", q);*/
    println!("Everything exited");
    Ok(())
}
