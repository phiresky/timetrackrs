use warp::Filter;

use rust_embed::RustEmbed;

use crate::prelude::*;
#[derive(RustEmbed)]
#[folder = "frontend/dist/"]
struct FrontendDistAssets;

pub async fn make_server() {
    let db = init_db_pool().await?;
    let index = warp::path!("/").map(|| include_str!("../../frontend/index.html"));

    warp::serve(index).run(([127, 0, 0, 1], 52714));
}
