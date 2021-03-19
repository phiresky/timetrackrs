use std::net::SocketAddr;

use warp::Filter;

use rust_embed::RustEmbed;

use crate::prelude::*;

use super::api_routes::api_routes;
#[derive(RustEmbed)]
#[folder = "frontend/dist/"]
struct FrontendDistAssets;

pub async fn make_server() -> anyhow::Result<()> {
    let db = init_db_pool().await?;
    let index = warp::path::end().map(|| include_str!("../../frontend/index.html"));

    let routes = index.or(warp::path("api").and(api_routes(db)));

    let listen: SocketAddr = "127.0.0.1:52714".parse()?;
    println!("starting server at {}", listen);
    warp::serve(routes).run(listen).await;

    Ok(())
}
