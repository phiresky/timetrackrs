#![feature(proc_macro_hygiene, decl_macro)]

use diesel::prelude::*;
use rocket::{get, post, routes};
use rocket_contrib::json::Json;
use serde_json::json;
use serde_json::Value as J;
use track_pc_usage_rs as trbtt;
use track_pc_usage_rs::events::deserialize_captured;
use track_pc_usage_rs::util::iso_string_to_date;
use trbtt::db::models::{DbEvent, Timestamptz};
use trbtt::extract::ExtractInfo;
use trbtt::prelude::*;
#[macro_use]
extern crate rocket_contrib;

#[database("events_database")]
struct DbConn(diesel::SqliteConnection);

type DebugRes<T> = Result<T, rocket::response::Debug<anyhow::Error>>;
#[get("/time-range?<after>&<before>&<limit>")]
fn time_range(
    mut db: DbConn,
    after: Option<String>,
    limit: Option<u32>,
    before: Option<String>,
) -> DebugRes<Json<J>> {
    // println!("handling...");
    // println!("querying...");
    let mdata = {
        use trbtt::db::schema::events::dsl::*;
        let mut query = events.into_boxed();
        if let Some(after) = after {
            let after = iso_string_to_date(&after)?;
            query = query
                .filter(timestamp.gt(Timestamptz::new(after)))
                .order(timestamp.asc());
        }
        if let Some(before) = before {
            let before = iso_string_to_date(&before)?;
            query = query
                .filter(timestamp.lt(Timestamptz::new(before)))
                .order(timestamp.desc());
        }
        let limit = limit.unwrap_or(100);
        query
            .limit(limit as i64)
            .load::<DbEvent>(&*db)
            .context("fetching from db")?
    };
    let mut dbsy = DatyBasy::new(&mut *db);
    // println!("jsonifying...");
    let v = mdata
        .into_iter()
        .filter_map(|a| {
            let r = deserialize_captured((&a.data_type, &a.data));
            match r {
                Ok(r) => {
                    if let Some(data) = r.extract_info() {
                        Some(json!({
                            "id": a.id,
                            "timestamp": a.timestamp,
                            "duration": a.sampler.get_duration(),
                            "tags": tags::get_tags(&mut dbsy, data).map_err(|e| {
                                log::warn!("get tags of {} error: {:?}", a.id, e);
                                e
                            }).ok()?,
                        }))
                    } else {
                        None
                    }
                }
                Err(e) => {
                    log::warn!("deser of {} error: {:?}", a.id, e);
                    // println!("data=||{}", a.data);
                    None
                }
            }
        })
        .collect::<Vec<_>>();
    Ok(Json(json!({ "data": &v })))
}

#[get("/single-event?<id>")]
fn single_event(mut db: DbConn, id: String) -> DebugRes<Json<J>> {
    // println!("handling...");
    // println!("querying...");
    let a = {
        use trbtt::db::schema::events::dsl;
        dsl::events
            .filter(dsl::id.eq(id))
            .first::<DbEvent>(&*db)
            .context("fetching from db")?
    };
    // println!("jsonifying...");
    let mut dbsy = DatyBasy::new(&mut *db);

    let r = deserialize_captured((&a.data_type, &a.data));
    let v = match r {
        Ok(r) => {
            if let Some(data) = r.extract_info() {
                Some(json!({
                    "id": a.id,
                    "timestamp": a.timestamp,
                    "duration": a.sampler.get_duration(),
                    "tags": get_tags(&mut dbsy, data)?,
                    "raw": r
                }))
            } else {
                None
            }
        }
        Err(e) => {
            println!("deser of {} error: {:?}", a.id, e);
            // println!("data=||{}", a.data);
            None
        }
    };

    Ok(Json(json!({ "data": &v })))
}

#[get("/rule-groups")]
fn rule_groups(db: DbConn) -> DebugRes<Json<J>> {
    // println!("handling...");
    // println!("querying...");
    use trbtt::db::schema::tag_rule_groups::dsl::*;
    let groups = tag_rule_groups
        .load::<TagRuleGroup>(&*db)
        .context("fetching from db")?;

    Ok(Json(json!({ "data": &groups })))
}

#[post("/rule-groups", format = "json", data = "<input>")]
fn update_rule_groups(db: DbConn, input: Json<Vec<TagRuleGroup>>) -> DebugRes<Json<J>> {
    // println!("handling...");
    // println!("querying...");
    use trbtt::db::schema::tag_rule_groups::dsl::*;
    db.transaction::<(), anyhow::Error, _>(|| {
        for g in input.into_inner() {
            let updated = diesel::update(tag_rule_groups)
                .set(&g)
                .execute(&*db)
                .context("updating in db")?;

            if updated == 0 {
                log::info!("inserting new group");
                diesel::insert_into(tag_rule_groups)
                    .values(g)
                    .execute(&*db)
                    .context("inserting into db")?;
            }
        }
        Ok(())
    })?;

    Ok(Json(json!({})))
}

fn main() -> anyhow::Result<()> {
    util::init_logging();
    dotenv::dotenv().ok();

    use rocket::config::{Config, Environment, Value};
    use std::collections::HashMap;

    let mut database_config = HashMap::new();
    let mut databases = HashMap::new();

    let database_url = trbtt::db::get_database_location();

    // This is the same as the following TOML:
    // my_db = { url = "database.sqlite" }
    database_config.insert("url", Value::from(database_url));
    databases.insert("events_database", Value::from(database_config));

    let config = Config::build(Environment::Development)
        .extra("databases", databases)
        .finalize()
        .unwrap();

    let cors = rocket_cors::CorsOptions {
        allowed_origins: rocket_cors::AllowedOrigins::all(),
        ..Default::default()
    }
    .to_cors()?;
    rocket::custom(config)
        .mount(
            "/api",
            routes![time_range, single_event, rule_groups, update_rule_groups],
        )
        .attach(cors)
        .attach(DbConn::fairing())
        .launch();

    Ok(())
}
