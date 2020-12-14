#![feature(proc_macro_hygiene, decl_macro)]

use std::time::Instant;

use diesel::prelude::*;
use rocket::{get, post, routes};
use rocket_contrib::json::Json;

use track_pc_usage_rs as trbtt;
use track_pc_usage_rs::events::deserialize_captured;
use track_pc_usage_rs::util::iso_string_to_date;
use trbtt::db::models::{DbEvent, Timestamptz};
use trbtt::extract::ExtractInfo;
use trbtt::prelude::*;
#[macro_use]
extern crate rocket_contrib;

use api::*;

#[get("/time-range?<after>&<before>&<limit>")]
fn time_range(
    db: DatyBasy,
    before: Option<String>,
    after: Option<String>,
    limit: Option<usize>,
) -> Api::time_range::response {
    // println!("handling...");
    // println!("querying...");
    let before = match before {
        Some(before) => Some(iso_string_to_date(&before).context("could not parse before date")?),
        None => None,
    };
    let after = match after {
        Some(after) => Some(iso_string_to_date(&after).context("could not parse after date")?),
        None => None,
    };

    // if we get events before date X then we need to sort descending
    // if both before and after are set either would be ok
    let mdata = YieldEventsFromTrbttDatabase {
        db: &db.db_events,
        chunk_size: 1000,
        last_fetched: Timestamptz::new(
            before
                .or(after)
                .context("one of after and before must be specified")?,
        ),
        ascending: before.is_none(),
    };

    // println!("jsonifying...");
    let now = Instant::now();

    let mut total_seen = 0;

    let v = mdata
        .into_iter()
        .flatten()
        .take_while(|a| match (before, after) {
            (Some(_before), Some(after)) => {
                // both limits exist, we are fetching descending,
                a.timestamp >= Timestamptz::new(after)
            }
            _ => true,
        })
        .filter_map(|a| {
            total_seen += 1;
            let r = deserialize_captured((&a.data_type, &a.data));
            match r {
                Ok(r) => {
                    if let Some(data) = r.extract_info() {
                        Some(SingleExtractedEvent {
                            id: a.id.clone(),
                            timestamp: a.timestamp.clone(),
                            duration: a.sampler.get_duration(),
                            tags: get_tags(&db, data)
                                .map_err(|e| {
                                    log::warn!("get tags of {} error: {:?}", a.id, e);
                                    e
                                })
                                .ok()?,
                        })
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
        .take(limit.unwrap_or(10000))
        .collect::<Vec<_>>();
    log::debug!(
        "after filter: {}/{}. extracting tags took {:?}",
        v.len(),
        total_seen,
        now.elapsed()
    );
    Ok(Json(ApiResponse { data: v }))
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

    let r = deserialize_captured((&a.data_type, &a.data));
    let v = match r {
        Ok(raw) => {
            if let Some(data) = raw.extract_info() {
                Some(SingleExtractedEventWithRaw {
                    id: a.id,
                    timestamp: a.timestamp,
                    duration: a.sampler.get_duration(),
                    tags_reasons: get_tags_with_reasons(&db, data.clone())?,
                    tags: get_tags(&db, data)?,
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
            routes![time_range, single_event, rule_groups, update_rule_groups],
        )
        .attach(cors)
        .attach(DbEvents::fairing())
        .attach(DbConfig::fairing())
        .attach(DbExtracted::fairing())
        .launch();

    Ok(())
}
