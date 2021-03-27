use std::path::PathBuf;

use futures::TryStreamExt;
use futures::{future::BoxFuture, never::Never, stream::FuturesUnordered};

use timetrackrs::util::init_logging;
use timetrackrs::{config::TimetrackrsConfig, prelude::*};

#[derive(StructOpt, Debug, Serialize, Deserialize)]
struct Args {
    #[structopt(long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();
    let args = Args::from_args();
    let db = init_db_pool().await?;
    let config: TimetrackrsConfig = match args.config {
        Some(path) => {
            serde_json::from_reader(File::open(path).context("Could not open config file")?)
                .context("Could not read config file")?
        }
        None => timetrackrs::config::default_config(),
    };

    println!("Configuration: {:#?}", config);

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
