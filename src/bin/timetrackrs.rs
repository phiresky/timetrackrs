use std::path::PathBuf;

use futures::{future::BoxFuture, never::Never, stream::FuturesUnordered};
use futures::{StreamExt, TryStreamExt};

use timetrackrs::{config::TimetrackrsConfig, prelude::*};
use timetrackrs::{db::clear_wal_files, util::init_logging};
use tokio::{
    task::{JoinError, JoinHandle},
    time::sleep,
};

#[derive(StructOpt, Debug, Serialize, Deserialize)]
struct Args {
    #[structopt(long)]
    config: Option<PathBuf>,
}

async fn cleanup_wal(db: DatyBasy) -> anyhow::Result<Never> {
    loop {
        if let Err(e) = clear_wal_files(&db.db)
            .await
            .context("Could not clear wal files")
        {
            log::warn!("{}", e);
        }
        sleep(Duration::from_secs(120)).await;
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    init_logging();
    let args = Args::from_args();
    let db = init_db_pool().await?;
    {
        // just get one connection to ensure pool is initialized (and migrated) ok
        db.db.acquire().await.context("Db Pool invalid")?;
    }
    let config: TimetrackrsConfig = match args.config {
        Some(path) => {
            serde_json::from_reader(File::open(path).context("Could not open config file")?)
                .context("Could not read config file")?
        }
        None => timetrackrs::config::default_config(),
    };

    println!("Configuration: {:#?}", config);

    let features: FuturesUnordered<JoinHandle<anyhow::Result<Never>>> = FuturesUnordered::new();

    for c in config.capturers {
        features.push(tokio::spawn(capture_loop(db.clone(), c)));
    }
    if let Some(server) = config.server {
        features.push(tokio::spawn(timetrackrs::server::server::run_server(
            db.clone(),
            server,
        )));
    }
    features.push(tokio::spawn(cleanup_wal(db.clone())));

    let mut features = features;

    while let Some(f) = features.next().await {
        f?.context("Some feature failed")?;
    }

    println!("Everything exited");
    Ok(())
}
