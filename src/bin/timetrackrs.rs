use track_pc_usage_rs::util::init_logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();
    track_pc_usage_rs::server::server::make_server().await?;
    /*let db = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite://foo.sqlite3")
        .await?;

    let q = sqlx::query!("select count(*) as coint from foo.events")
        .fetch_one(&db)
        .await?;
    println!("qq = {:?}", q);*/

    Ok(())
}
