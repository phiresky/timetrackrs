pub mod models;
pub mod schema;
use anyhow::Context;
use diesel::prelude::*;
use diesel_migrations::embed_migrations;
use dotenv::dotenv;
use std::env;
embed_migrations!();

pub fn connect() -> anyhow::Result<SqliteConnection> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = SqliteConnection::establish(&database_url).context("Establishing connection")?;
    db.execute("pragma page_size = 32768;")
        .context("setup pragma 1")?;
    db.execute("pragma foreign_keys = on;")
        .context("setup pragma 2")?;
    db.execute("pragma temp_store = memory;")
        .context("setup pragma 3")?;
    db.execute("pragma journal_mode = WAL;")
        .context("setup pragma 4")?;
    db.execute("pragma synchronous = normal;")
        .context("setup pragma 5")?;
    db.execute("pragma mmap_size = 30000000000;")
        .context("setup pragma 6")?;
    //db.execute("pragma auto_vacuum = incremental")
    //    .context("setup pragma 7")?;
    // db.execute("pragma optimize;")?;

    embedded_migrations::run_with_output(&db, &mut std::io::stdout()).context("migrations")?;
    Ok(db)
}
