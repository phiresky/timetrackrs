pub mod datybasy;
pub mod db_iterator;
pub mod models;
use anyhow::Context;
use sqlx::{sqlite::SqliteConnectOptions, Executor};
use sqlx::{sqlite::SqlitePoolOptions, SqliteConnection, SqlitePool};
use std::{env, path::PathBuf};

pub async fn connect() -> anyhow::Result<SqlitePool> {
    let dir = get_database_dir_location();
    let dir = dir.to_string_lossy().to_string();
    let main = format!("{}/lock.sqlite3", dir);
    log::debug!("Connecting to db at {}", dir);
    let db = SqlitePoolOptions::new()
        .after_connect(move |conn| {
            let dir = dir.clone();
            Box::pin(async move {
                for attachdb in &["raw_events", "extracted", "config"] {
                    let connn: &mut SqliteConnection = conn;
                    sqlx::query(&format!(
                        "ATTACH DATABASE '{}/{}.sqlite3' as {}",
                        dir, attachdb, attachdb
                    ))
                    .execute(connn)
                    .await?;
                }
                // attach
                set_pragmas(conn)
                    .await
                    .with_context(|| format!("set pragmas for db"))
                    .map_err(|e| {
                        /*let b: Box<(dyn std::error::Error + Sync + std::marker::Send + 'static)> =
                        Box::new(std::error::Error::from(e));*/
                        sqlx::error::Error::Configuration(e.into())
                    })?;
                Ok(())
            })
        })
        .connect_with(
            SqliteConnectOptions::new()
                .filename(&main)
                .create_if_missing(true),
        )
        .await
        .with_context(|| format!("Establishing connection to db {}", main))?;
    sqlx::migrate!()
        .run(&db)
        .await
        .context("running migrations")?;
    Ok(db)
}
pub async fn set_pragmas(db: &mut SqliteConnection) -> anyhow::Result<()> {
    let want_page_size = 32768;

    db.execute("pragma busy_timeout = 5000;")
        .await
        .context("setup pragma 1")?;
    db.execute(format!("pragma page_size = {};", want_page_size).as_str())
        .await
        .context("setup pragma 1")?;
    db.execute("pragma foreign_keys = on;")
        .await
        .context("setup pragma 2")?;
    db.execute("pragma temp_store = memory;")
        .await
        .context("setup pragma 3")?;
    db.execute("pragma journal_mode = WAL;")
        .await
        .context("setup pragma 4")?;
    db.execute("pragma wal_autocheckpoint = 20;")
        .await
        .context("setup pragma 4b")?;
    db.execute("pragma synchronous = normal;")
        .await
        .context("setup pragma 5")?;
    db.execute("pragma mmap_size = 30000000000;")
        .await
        .context("setup pragma 6")?;
    let dbb: &mut SqliteConnection = db;
    let page_size: i64 = sqlx::query_scalar("pragma page_size;")
        .fetch_one(dbb)
        .await?;
    let dbb: &mut SqliteConnection = db;
    let journal_mode: String = sqlx::query_scalar("pragma journal_mode;")
        .fetch_one(dbb)
        .await?;
    if page_size != want_page_size || journal_mode != "wal" {
        log::info!("vaccuuming db to ensure page size and journal mode");
        let dbb: &mut SqliteConnection = db;
        sqlx::query("pragma journal_mode = DELETE;")
            .execute(dbb)
            .await
            .context("setup pragma 7")?;
        let dbb: &mut SqliteConnection = db;
        sqlx::query("vacuum")
            .execute(dbb)
            .await
            .context("setup pragma 8")?;
        let dbb: &mut SqliteConnection = db;
        sqlx::query("pragma journal_mode = WAL;")
            .execute(dbb)
            .await
            .context("setup pragma 9")?;
    }
    let dbb: &mut SqliteConnection = db;
    sqlx::query("pragma auto_vacuum = full")
        .execute(dbb)
        .await
        .context("setup pragma 10")?;
    // db.execute("pragma optimize;")?;
    Ok(())
}

pub fn get_database_dir_location() -> PathBuf {
    let dir = env::var("TIMETRACKRS_DATA_DIR").unwrap_or_else(|_| {
        let dirs = directories_next::ProjectDirs::from("", "", "timetrackrs")
            .expect("No HOME directory found");
        let dir = dirs.data_dir();
        std::fs::create_dir_all(dir).expect("could not create data dir");
        dir.to_string_lossy().into()
    });
    PathBuf::from(dir)
}
