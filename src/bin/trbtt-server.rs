#![feature(proc_macro_hygiene, decl_macro)]

use std::{collections::HashSet, ffi::OsStr, io::Cursor, path::PathBuf, time::Instant};

use diesel::prelude::*;
use rocket::{
    get,
    http::{ContentType, Status}, response, routes,
};



use rust_embed::RustEmbed;
use track_pc_usage_rs as trbtt;



use trbtt::prelude::*;
#[macro_use]
extern crate rocket_contrib;



#[derive(RustEmbed)]
#[folder = "frontend/dist/"]
struct FrontendDistAssets;

#[get("/")]
fn index<'r>() -> response::Result<'r> {
    let data = Some(include_bytes!("../../frontend/index.html"));
    data.map_or_else(
        || Err(Status::NotFound),
        |d| {
            response::Response::build()
                .header(ContentType::HTML)
                .sized_body(Cursor::new(d))
                .ok()
        },
    )
}

#[rocket::catch(404)]
fn not_found<'r>() -> response::Result<'r> {
    index()
}

#[get("/dist/<file..>")]
fn dist<'r>(file: PathBuf) -> response::Result<'r> {
    let filename = file.display().to_string();
    FrontendDistAssets::get(&filename).map_or_else(
        || Err(Status::NotFound),
        |d| {
            let ext = file
                .as_path()
                .extension()
                .and_then(OsStr::to_str)
                .ok_or_else(|| Status::new(400, "Could not get file extension"))?;
            let content_type = ContentType::from_extension(ext).unwrap_or(ContentType::Binary);
            response::Response::build()
                .header(content_type)
                .sized_body(Cursor::new(d))
                .ok()
        },
    )
}

fn main() -> anyhow::Result<()> {
    util::init_logging();
    dotenv::dotenv().ok();

    use rocket::config::{Config, Environment, Value};
    use std::collections::HashMap;

    let mut databases = HashMap::new();

    let _database_url = trbtt::db::get_database_dir_location();

    // This is the same as the following TOML:
    // my_db = { url = "database.sqlite" }

    databases.insert(
        "raw_events_database",
        Value::from({
            let mut database_config = HashMap::new();
            database_config.insert("url", Value::from(trbtt::db::raw_events::get_filename()));
            database_config
        }),
    );
    databases.insert(
        "config_database",
        Value::from({
            let mut database_config = HashMap::new();
            database_config.insert("url", Value::from(trbtt::db::config::get_filename()));
            database_config
        }),
    );
    databases.insert(
        "extracted_database",
        Value::from({
            let mut database_config = HashMap::new();
            database_config.insert("url", Value::from(trbtt::db::extracted::get_filename()));
            database_config
        }),
    );

    let config = Config::build(Environment::Development)
        .port(52714)
        .extra("databases", databases)
        .finalize()
        .unwrap();

    // TODO: remove in prod
    let cors = rocket_cors::CorsOptions {
        allowed_origins: rocket_cors::AllowedOrigins::some_exact(&[
            "http://localhost:8081",
            "http://localhost:8080",
        ]),
        ..Default::default()
    }
    .to_cors()?;
    
    rocket::custom(config)
        .register(rocket::catchers![not_found])
        .mount("/", routes![index, dist])
        /*.mount(
            "/api",
            routes![
                get_known_tags,
                time_range,
                single_event,
                rule_groups,
                update_rule_groups
            ],
        )*/
        .attach(cors)
        .attach(DbEvents::fairing())
        .attach(DbConfig::fairing())
        .attach(DbExtracted::fairing())
        .launch();

    Ok(())
}
