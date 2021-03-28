pub mod datybasy;
pub mod db_iterator;
pub mod models;
use anyhow::Context;
use sqlx::{sqlite::SqliteConnectOptions, Executor};
use sqlx::{sqlite::SqlitePoolOptions, SqliteConnection, SqlitePool};
use std::{env, path::PathBuf};

pub async fn connect(pool_size: Option<u32>) -> anyhow::Result<SqlitePool> {
    let dir = get_database_dir_location();
    let dir = dir.to_string_lossy().to_string();
    let main = format!("{}/lock.sqlite3", dir);
    log::debug!("Connecting to db at {}", dir);
    let db = SqlitePoolOptions::new()
        .max_connections(pool_size.unwrap_or(1))
        .after_connect(move |conn| {
            let dir = dir.clone();
            Box::pin(async move {
                set_pragmas(conn, None)
                    .await
                    .with_context(|| "set pragmas for root")
                    .map_err(|e| {
                        /*let b: Box<(dyn std::error::Error + Sync + std::marker::Send + 'static)> =
                        Box::new(std::error::Error::from(e));*/
                        sqlx::error::Error::Configuration(e.into())
                    })?;
                for attachdb in &["raw_events", "extracted", "config"] {
                    let connn: &mut SqliteConnection = conn;
                    sqlx::query(&format!(
                        "ATTACH DATABASE '{}/{}.sqlite3' as {}",
                        dir, attachdb, attachdb
                    ))
                    .execute(connn)
                    .await?;
                    set_pragmas(conn, Some(attachdb))
                        .await
                        .with_context(|| format!("set pragmas for {}", attachdb))
                        .map_err(|e| {
                            /*let b: Box<(dyn std::error::Error + Sync + std::marker::Send + 'static)> =
                            Box::new(std::error::Error::from(e));*/
                            sqlx::error::Error::Configuration(e.into())
                        })?;
                }

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
    let migrator = sqlx::migrate!();
    log::info!("Running {} migrations", migrator.iter().count());
    migrator.run(&db).await.context("running migrations")?;
    Ok(db)
}
pub async fn set_pragmas(db: &mut SqliteConnection, schema: Option<&str>) -> anyhow::Result<()> {
    let want_page_size = 32768;
    let prefix = schema.map(|s| format!("{}.", s)).unwrap_or_default();

    db.execute(format!("pragma {}busy_timeout = 5000;", prefix).as_str())
        .await
        .context("setup pragma 1")?;
    db.execute(format!("pragma {}page_size = {};", prefix, want_page_size).as_str())
        .await
        .context("setup pragma 1")?;
    db.execute(format!("pragma {}foreign_keys = on;", prefix).as_str())
        .await
        .context("setup pragma 2")?;
    db.execute(format!("pragma {}temp_store = memory;", prefix).as_str())
        .await
        .context("setup pragma 3")?;
    db.execute(format!("pragma {}journal_mode = WAL;", prefix).as_str())
        .await
        .context("setup pragma 4")?;
    db.execute(format!("pragma {}wal_autocheckpoint = 20;", prefix).as_str())
        .await
        .context("setup pragma 4b")?;
    db.execute(format!("pragma {}synchronous = normal;", prefix).as_str())
        .await
        .context("setup pragma 5")?;
    db.execute(format!("pragma {}mmap_size = 30000000000;", prefix).as_str())
        .await
        .context("setup pragma 6")?;
    let dbb: &mut SqliteConnection = db;
    let page_size: i64 = sqlx::query_scalar(format!("pragma {}page_size;", prefix).as_str())
        .fetch_one(dbb)
        .await?;
    let dbb: &mut SqliteConnection = db;
    let journal_mode: String =
        sqlx::query_scalar(format!("pragma {}journal_mode;", prefix).as_str())
            .fetch_one(dbb)
            .await?;
    if page_size != want_page_size || journal_mode != "wal" {
        log::info!("vaccuuming db to ensure page size and journal mode");
        let dbb: &mut SqliteConnection = db;
        sqlx::query(format!("pragma {}journal_mode = DELETE;", prefix).as_str())
            .execute(dbb)
            .await
            .context("setup pragma 7")?;
        let dbb: &mut SqliteConnection = db;
        sqlx::query(format!("vacuum {}", schema.unwrap_or_default()).as_str())
            .execute(dbb)
            .await
            .context("setup pragma 8")?;
        let dbb: &mut SqliteConnection = db;
        sqlx::query(format!("pragma {}journal_mode = WAL;", prefix).as_str())
            .execute(dbb)
            .await
            .context("setup pragma 9")?;
    }
    let dbb: &mut SqliteConnection = db;
    sqlx::query(format!("pragma {}auto_vacuum = full", prefix).as_str())
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
