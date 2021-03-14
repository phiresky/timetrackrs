pub mod datybasy;
pub mod db_iterator;
pub mod hack;
pub mod models;
#[allow(clippy::all)]
pub mod schema;
use anyhow::Context;
use diesel::sql_types::{BigInt, Text};
use sqlx::Executor;
use sqlx::{sqlite::SqlitePoolOptions, SqliteConnection, SqlitePool};
use std::{env, fmt::Display, path::PathBuf};

pub async fn connect() -> anyhow::Result<SqlitePool> {
    let dir = get_database_dir_location().to_string_lossy();
    let main = format!("{}/lock.sqlite3", dir);
    log::debug!("Connecting to db at {}", dir);
    let db = SqlitePoolOptions::new()
        .after_connect(|conn| {
            Box::pin(async move {
                // attach
                set_pragmas(conn)
                    .await
                    .with_context(|| format!("set pragmas for db"))
                    .map_err(|e| sqlx::error::Error::Configuration(Box::new(e.into())))?;
                Ok(())
            })
        })
        .connect(&main)
        .await
        .context("Establishing connection")?;
    sqlx::migrate!()
        .run(&db)
        .await
        .context("running migrations")?;
    Ok(db)
}

#[derive(Debug, QueryableByName)]
struct P {
    #[sql_type = "BigInt"]
    page_size: i64,
}
#[derive(Debug, QueryableByName)]
struct P2 {
    #[sql_type = "Text"]
    journal_mode: String,
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
    let dir = env::var("TRBTT_DATA_DIR").unwrap_or_else(|_| {
        let dirs =
            directories_next::ProjectDirs::from("", "", "trbtt").expect("No HOME directory found");
        let dir = dirs.data_dir();
        std::fs::create_dir_all(dir).expect("could not create data dir");
        dir.to_string_lossy().into()
    });
    PathBuf::from(dir)
}
