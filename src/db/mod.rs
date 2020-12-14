pub mod datybasy;
pub mod db_iterator;
pub mod hack;
pub mod models;
#[allow(clippy::all)]
pub mod schema;
use anyhow::Context;
use diesel::prelude::*;
use std::{env, path::PathBuf};

macro_rules! kot {
    () => {
        use super::*;
        use diesel::SqliteConnection;
        use diesel_migrations::embed_migrations;
        pub fn migrate(conn: &SqliteConnection) -> anyhow::Result<()> {
            Ok(embedded_migrations::run_with_output(
                conn,
                &mut std::io::stdout(),
            )?)
        }
        pub fn connect_file(filename: &str) -> anyhow::Result<SqliteConnection> {
            let db = SqliteConnection::establish(&filename).context("Establishing connection")?;
            set_pragmas(&db).with_context(|| format!("set pragmas for {}", &filename))?;
            migrate(&db).context("run migrations")?;
            Ok(db)
        }
        pub fn get_filename() -> String {
            let mut file = get_database_dir_location();
            file.push(format!("{}.sqlite3", DB_NAME));
            log::info!("get filename {:?}", file);
            file.to_string_lossy().to_string()
        }

        pub fn connect() -> anyhow::Result<SqliteConnection> {
            let file = get_filename();
            log::debug!("Connecting to db at {}", file);
            connect_file(&file)
        }
    };
}

pub mod raw_events {
    static DB_NAME: &str = "raw_events";
    embed_migrations!("diesel/raw_events_migrations");
    kot! {}
}
pub mod config {
    static DB_NAME: &str = "config";
    embed_migrations!("diesel/config_migrations");
    kot! {}
}
pub mod extracted {
    static DB_NAME: &str = "extracted";
    embed_migrations!("diesel/extracted_migrations");
    kot! {}
}

pub fn set_pragmas(db: &SqliteConnection) -> anyhow::Result<()> {
    db.execute("pragma page_size = 32768;")
        .context("setup pragma 1")?;
    db.execute("pragma foreign_keys = on;")
        .context("setup pragma 2")?;
    db.execute("pragma temp_store = memory;")
        .context("setup pragma 3")?;
    db.execute("pragma journal_mode = WAL;")
        .context("setup pragma 4")?;
    db.execute("pragma wal_autocheckpoint = 20;")
        .context("setup pragma 4b")?;
    db.execute("pragma synchronous = normal;")
        .context("setup pragma 5")?;
    db.execute("pragma mmap_size = 30000000000;")
        .context("setup pragma 6")?;
    //db.execute("pragma auto_vacuum = incremental")
    //    .context("setup pragma 7")?;
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
