use std::net::SocketAddr;

use futures::never::Never;
use hyper::StatusCode;
use warp::Filter;

use crate::prelude::*;
use rust_embed::RustEmbed;
use warp::{http::header::HeaderValue, path::Tail, reply::Response, Rejection, Reply};

use super::api_routes::{api_routes, ErrAsJson};

pub struct ServerConfig {
    pub listen: Vec<String>,
}

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

async fn handle_error(rej: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(err) = rej.find::<ErrAsJson>() {
        let reply = err.to_json();
        //reply.set_status
        return Ok(warp::reply::with_status(
            reply,
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }
    Err(rej)
}

pub async fn run_server(db: DatyBasy, config: ServerConfig) -> anyhow::Result<Never> {
    let index = warp::path::end()
        .or(warp::path("plot"))
        .or(warp::path("timeline"))
        .map(|_| warp::reply::html(include_str!("../../frontend/index.html")));

    let static_files = warp::path("dist")
        .and(warp::path::tail())
        .and_then(serve_static);

    let routes = index
        .or(static_files)
        .or(warp::path("api").and(api_routes(db)))
        .recover(handle_error);

    let futures = config.listen.iter().map(|listen: &String| {
        println!("starting server at {}", listen);
        let listen = listen.to_string();
        let routes = routes.clone();
        async move {
            let (_, fut) = warp::serve(routes)
                .try_bind_ephemeral(
                    listen
                        .parse::<SocketAddr>()
                        .with_context(|| format!("Could not parse listen address {}", listen))?,
                )
                .context("Could not bind to address")?;
            fut.await;
            Ok::<_, anyhow::Error>(())
        }
    });

    futures::future::try_join_all(futures)
        .await
        .context("Could not create server")?;

    anyhow::bail!("should never return??")
}
