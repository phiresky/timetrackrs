pub mod caching_int_map;
pub mod datybasy;
pub mod db_iterator;
pub mod models;
pub use crate::prelude::*;
use anyhow::Context;
use sqlx::{sqlite::SqliteConnectOptions, Executor};
use sqlx::{sqlite::SqlitePoolOptions, SqliteConnection, SqlitePool};
use std::{env, path::PathBuf};

fn dbs_list() -> &'static [&'static str] {
    &["raw_events", "extracted", "config"]
}

pub async fn clear_wal_files(db: &SqlitePool) -> anyhow::Result<()> {
    log::debug!("running wal_checkpoint(truncate)");
    for attachdb in dbs_list() {
        sqlx::query(&format!("pragma {attachdb}.wal_checkpoint(truncate);"))
            .execute(db)
            .await?;
    }
    Ok(())
}
pub async fn connect(pool_size: Option<u32>) -> anyhow::Result<SqlitePool> {
    connect_dir(
        get_database_dir_location().to_string_lossy().to_string(),
        pool_size,
    )
    .await
}
pub async fn connect_dir(dir: String, pool_size: Option<u32>) -> anyhow::Result<SqlitePool> {
    let main = format!("{dir}/lock.sqlite3");
    log::debug!("Connecting to db at {}", dir);
    let db = SqlitePoolOptions::new()
        .max_connections(pool_size.unwrap_or(10))
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
                for attachdb in dbs_list() {
                    let connn: &mut SqliteConnection = conn;
                    sqlx::query(&format!(
                        "ATTACH DATABASE '{dir}/{attachdb}.sqlite3' as {attachdb}"
                    ))
                    .execute(connn)
                    .await?;
                    set_pragmas(conn, Some(attachdb))
                        .await
                        .with_context(|| format!("set pragmas for {attachdb}"))
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
        .with_context(|| format!("Establishing connection to db {main}"))?;
    let migrator = sqlx::migrate!();
    log::info!("Running {} migrations ", migrator.iter().count());
    migrator.run(&db).await.context("running migrations")?;
    Ok(db)
}
pub async fn set_pragmas(db: &mut SqliteConnection, schema: Option<&str>) -> anyhow::Result<()> {
    let want_page_size = 32768;
    let prefix = schema.map(|s| format!("{s}.")).unwrap_or_default();

    db.execute(format!("pragma {prefix}busy_timeout = 5000;").as_str())
        .await
        .context("setup pragma 1")?;
    db.execute(format!("pragma {prefix}page_size = {want_page_size};").as_str())
        .await
        .context("setup pragma 1")?;
    db.execute(format!("pragma {prefix}foreign_keys = on;").as_str())
        .await
        .context("setup pragma 2")?;
    db.execute(format!("pragma {prefix}temp_store = memory;").as_str())
        .await
        .context("setup pragma 3")?;
    db.execute(format!("pragma {prefix}journal_mode = WAL;").as_str())
        .await
        .context("setup pragma 4")?;
    db.execute(format!("pragma {prefix}wal_autocheckpoint = 20;").as_str())
        .await
        .context("setup pragma 4b")?;
    db.execute(format!("pragma {prefix}synchronous = normal;").as_str())
        .await
        .context("setup pragma 5")?;
    db.execute(format!("pragma {prefix}mmap_size = 30000000000;").as_str())
        .await
        .context("setup pragma 6")?;
    let dbb: &mut SqliteConnection = db;
    let page_size: i64 = sqlx::query_scalar(format!("pragma {prefix}page_size;").as_str())
        .fetch_one(dbb)
        .await?;
    let dbb: &mut SqliteConnection = db;
    let journal_mode: String =
        sqlx::query_scalar(format!("pragma {prefix}journal_mode;").as_str())
            .fetch_one(dbb)
            .await?;
    if page_size != want_page_size || journal_mode != "wal" {
        log::info!("vaccuuming db to ensure page size and journal mode");
        let dbb: &mut SqliteConnection = db;
        sqlx::query(format!("pragma {prefix}journal_mode = DELETE;").as_str())
            .execute(dbb)
            .await
            .context("setup pragma 7")?;
        let dbb: &mut SqliteConnection = db;
        sqlx::query(format!("vacuum {}", schema.unwrap_or_default()).as_str())
            .execute(dbb)
            .await
            .context("setup pragma 8")?;
        let dbb: &mut SqliteConnection = db;
        sqlx::query(format!("pragma {prefix}journal_mode = WAL;").as_str())
            .execute(dbb)
            .await
            .context("setup pragma 9")?;
    }
    let dbb: &mut SqliteConnection = db;
    sqlx::query(format!("pragma {prefix}auto_vacuum = full").as_str())
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
