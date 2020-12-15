pub mod datybasy;
pub mod db_iterator;
pub mod hack;
pub mod models;
#[allow(clippy::all)]
pub mod schema;
use anyhow::Context;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Text};
use std::{env, path::PathBuf};
macro_rules! kot {
    () => {
        use super::*;
        use diesel::SqliteConnection;
        use diesel_migrations::embed_migrations;
        fn migrate(conn: &SqliteConnection) -> anyhow::Result<()> {
            Ok(embedded_migrations::run_with_output(
                conn,
                &mut std::io::stdout(),
            )?)
        }
        pub fn set_pragmas_migrate(db: &SqliteConnection) -> anyhow::Result<()> {
            set_pragmas(&db).with_context(|| format!("set pragmas for db"))?;
            migrate(&db).context("run migrations")?;
            Ok(())
        }
        pub fn connect_file(filename: &str) -> anyhow::Result<SqliteConnection> {
            let db = SqliteConnection::establish(&filename).context("Establishing connection")?;
            set_pragmas_migrate(&db)?;
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
pub fn set_pragmas(db: &SqliteConnection) -> anyhow::Result<()> {
    let want_page_size = 32768;
    db.execute(&format!("pragma page_size = {};", want_page_size))
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
    let P { page_size } = diesel::sql_query("pragma page_size;").get_result(db)?;
    let P2 { journal_mode } = diesel::sql_query("pragma journal_mode;").get_result(db)?;
    if page_size != want_page_size || journal_mode != "wal" {
        log::info!("vaccuuming db to ensure page size and journal mode");
        db.execute("pragma journal_mode = DELETE;")?;
        db.execute("vacuum")?;
        db.execute("pragma journal_mode = WAL;")?;
    }
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
