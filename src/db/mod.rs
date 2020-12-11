pub mod datybasy;
pub mod db_iterator;
pub mod hack;
pub mod models;
pub mod schema;
use anyhow::Context;
use diesel::prelude::*;
use diesel_migrations::embed_migrations;
use dotenv::dotenv;
use std::{
    env,
};
embed_migrations!();

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
pub fn connect_file(filename: &str) -> anyhow::Result<SqliteConnection> {
    let db = SqliteConnection::establish(&filename).context("Establishing connection")?;
    set_pragmas(&db).with_context(|| format!("set pragmas for {}", &filename))?;
    embedded_migrations::run_with_output(&db, &mut std::io::stdout()).context("migrations")?;
    Ok(db)
}
pub fn get_database_location() -> String {
    let database_location = env::var("DATABASE_URL").unwrap_or_else(|_| {
        let dirs =
            directories_next::ProjectDirs::from("", "", "trbtt").expect("No HOME directory found");
        let dir = dirs.data_dir();
        std::fs::create_dir_all(dir).expect("could not create data dir");
        dir.join("events.sqlite3")
            .to_str()
            .expect("user data dir is invalid unicode")
            .to_string()
    });
    database_location
}

pub fn connect() -> anyhow::Result<SqliteConnection> {
    dotenv().ok();
    let database_location = get_database_location();
    log::debug!("Connecting to db at {}", database_location);
    connect_file(&database_location)
}
