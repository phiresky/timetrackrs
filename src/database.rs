use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

use diesel_migrations::embed_migrations;
embed_migrations!();

pub fn connect() -> anyhow::Result<SqliteConnection> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = SqliteConnection::establish(&database_url)?;
    db.execute("pragma page_size = 32768;")?;
    db.execute("pragma foreign_keys = on;")?;
    db.execute("pragma temp_store = memory;")?;
    db.execute("pragma journal_mode = WAL;")?;
    db.execute("pragma synchronous = normal;")?;
    db.execute("pragma mmap_size= 30000000000;")?;
    db.execute("pragma auto_vacuum = incremental")?;
    // db.execute("pragma optimize;")?;

    embedded_migrations::run_with_output(&db, &mut std::io::stdout())?;
    Ok(db)
}
