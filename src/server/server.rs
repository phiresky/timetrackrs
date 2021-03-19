use std::net::SocketAddr;

use warp::Filter;

use crate::prelude::*;
use rust_embed::RustEmbed;
use warp::{http::header::HeaderValue, path::Tail, reply::Response, Rejection, Reply};

use super::api_routes::api_routes;
#[derive(RustEmbed)]
#[folder = "frontend/dist/"]
struct FrontendDistAssets;

async fn serve_static(path: Tail) -> Result<impl Reply, Rejection> {
    let path = path.as_str();
    let asset = FrontendDistAssets::get(path).ok_or_else(warp::reject::not_found)?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let mut res = Response::new(asset.into());
    res.headers_mut().insert(
        "content-type",
        HeaderValue::from_str(mime.as_ref()).unwrap(),
    );
    Ok(res)
}

pub async fn make_server() -> anyhow::Result<()> {
    let db = init_db_pool().await?;
    let index =
        warp::path::end().map(|| warp::reply::html(include_str!("../../frontend/index.html")));

    let static_files = warp::path("dist")
        .and(warp::path::tail())
        .and_then(serve_static);

    let routes = index
        .or(static_files)
        .or(warp::path("api").and(api_routes(db)));

    let listen: SocketAddr = "127.0.0.1:52714".parse()?;
    println!("starting server at {}", listen);
    warp::serve(routes).run(listen).await;

    Ok(())
}
