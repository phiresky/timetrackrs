use futures::TryStreamExt;
use futures::{future::BoxFuture, never::Never, stream::FuturesUnordered};

use timetrackrs::prelude::*;
use timetrackrs::util::init_logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();
    let db = init_db_pool().await?;
    let config = timetrackrs::config::default_config();

    let features: FuturesUnordered<BoxFuture<anyhow::Result<Never>>> = FuturesUnordered::new();

    for c in config.capturers {
        features.push(Box::pin(capture_loop(db.clone(), c)));
    }
    if let Some(server) = config.server {
        features.push(Box::pin(timetrackrs::server::server::run_server(
            db.clone(),
            server,
        )));
    }

    let _results: Vec<_> = features
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
