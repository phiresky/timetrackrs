#![feature(proc_macro_hygiene, decl_macro)]

use std::{collections::HashSet, time::Instant};

use diesel::prelude::*;
use rocket::{get, post, routes};
use rocket_contrib::json::Json;

use track_pc_usage_rs as trbtt;
use track_pc_usage_rs::util::iso_string_to_datetime;
use trbtt::db::models::{DbEvent, Timestamptz};
use trbtt::extract::ExtractInfo;
use trbtt::prelude::*;
#[macro_use]
extern crate rocket_contrib;

use api::*;

#[get("/get-known-tags")]
fn get_known_tags(db: DatyBasy) -> Api::get_known_tags::response {
    let mdata = YieldEventsFromTrbttDatabase {
        db: &db.db_events,
        chunk_size: 10000,
        last_fetched: Timestamptz(Utc::now()),
        ascending: false,
    };
    let s: HashSet<String> = mdata
        .take(1)
        .flatten()
        .flat_map(|e| -> Vec<String> {
            let data = e.deserialize_data();
            if let Ok(d) = data {
                let e = d.extract_info();
                if let Some(o) = e {
                    return o.into_iter().map(|(tag, _)| tag).collect();
                }
            }
            vec![]
        })
        .collect();

    let mut o = Vec::new();
    o.extend(s);
    Ok(Json(ApiResponse { data: o }))
}

#[get("/time-range?<after>&<before>")]
fn time_range(db: DatyBasy, before: String, after: String) -> Api::time_range::response {
    // println!("handling...");
    // println!("querying...");
    let before = iso_string_to_datetime(&before).context("could not parse before date")?;
    let after = iso_string_to_datetime(&after).context("could not parse after date")?;

    Ok(Json(ApiResponse {
        data: db
            .get_extracted_for_time_range(&Timestamptz(after), &Timestamptz(before))
            .context("get extracted events")?,
    }))
}

#[get("/single-event?<id>")]
fn single_event(db: DatyBasy, id: String) -> Api::single_event::response {
    // println!("handling...");
    // println!("querying...");
    let a = {
        use trbtt::db::schema::raw_events::events::dsl;
        dsl::events
            .filter(dsl::id.eq(id))
            .first::<DbEvent>(&*db.db_events)
            .context("fetching from db")?
    };

    let r = a.deserialize_data();
    let v = match r {
        Ok(raw) => {
            if let Some(data) = raw.extract_info() {
                Some(SingleExtractedEventWithRaw {
                    id: a.id,
                    timestamp: a.timestamp,
                    duration: a.sampler.get_duration(),
                    tags_reasons: get_tags_with_reasons(&db, data.clone())?,
                    tags: get_tags(&db, data),
                    raw,
                })
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

    Ok(Json(ApiResponse { data: v }))
}

#[get("/rule-groups")]
fn rule_groups(db: DatyBasy) -> Api::rule_groups::response {
    // println!("handling...");
    // println!("querying...");
    use trbtt::db::schema::config::tag_rule_groups::dsl::*;
    let groups = tag_rule_groups
        .load::<TagRuleGroup>(&*db.db_config)
        .context("fetching from db")?
        .into_iter()
        .chain(get_default_tag_rule_groups())
        .collect::<Vec<_>>();

    Ok(Json(ApiResponse { data: groups }))
}

#[post("/rule-groups", format = "json", data = "<input>")]
fn update_rule_groups(
    db: DatyBasy,
    input: Json<Vec<TagRuleGroup>>,
) -> Api::update_rule_groups::response {
    // println!("handling...");
    // println!("querying...");
    use trbtt::db::schema::config::tag_rule_groups::dsl::*;
    db.db_config.transaction::<(), anyhow::Error, _>(|| {
        for g in input.into_inner() {
            let q = diesel::update(&g).set(&g);
            log::info!("query: {}", diesel::debug_query(&q));
            let updated = q.execute(&*db.db_config).context("updating in db")?;

            if updated == 0 {
                log::info!("inserting new group");
                diesel::insert_into(tag_rule_groups)
                    .values(g)
                    .execute(&*db.db_config)
                    .context("inserting into db")?;
            }
        }
        Ok(())
    })?;

    Ok(Json(ApiResponse { data: () }))
}

fn main() -> anyhow::Result<()> {
    util::init_logging();
    dotenv::dotenv().ok();

    use rocket::config::{Config, Environment, Value};
    use std::collections::HashMap;

    let mut databases = HashMap::new();

    let database_url = trbtt::db::get_database_dir_location();

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
        .mount(
            "/api",
            routes![
                get_known_tags,
                time_range,
                single_event,
                rule_groups,
                update_rule_groups
            ],
        )
        .attach(cors)
        .attach(DbEvents::fairing())
        .attach(DbConfig::fairing())
        .attach(DbExtracted::fairing())
        .launch();

    Ok(())
}
