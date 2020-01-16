#![feature(proc_macro_hygiene, decl_macro)]

use chrono::{DateTime, Local};
use diesel::prelude::*;
use rocket::{get, routes};
use rocket_contrib::json::Json;
use rocket_cors::Responder;
use serde_json::json;
use serde_json::Value as J;
use track_pc_usage_rs as trbtt;
use track_pc_usage_rs::capture::deserialize_captured;
use track_pc_usage_rs::capture::CapturedData;
use trbtt::extract::properties::extract_info;
use trbtt::models::{Activity, Timestamptz};
#[macro_use]
extern crate rocket_contrib;

#[database("activity_database")]
struct DbConn(diesel::SqliteConnection);

#[get("/fetch-activity/<from>/<to>")]
fn fetch_activity(db: DbConn, from: String, to: String) -> anyhow::Result<Json<J>> {
    // println!("handling...");
    let from =
        DateTime::<chrono::FixedOffset>::parse_from_rfc3339(&from)?.with_timezone(&chrono::Utc);
    let to = DateTime::<chrono::FixedOffset>::parse_from_rfc3339(&to)?.with_timezone(&chrono::Utc);

    use trbtt::schema::activity::dsl::*;
    // println!("querying...");
    let mdata = activity
        .filter(timestamp.ge(Timestamptz::new(from)))
        .filter(timestamp.lt(Timestamptz::new(to)))
        .load::<Activity>(&*db)?;
    // println!("jsonifying...");
    let v = mdata
        .into_iter()
        .map(|a| {
            Ok(json!({
                "id": a.id,
                "timestamp": a.timestamp,
                "data_type": a.data_type,
                "data_type_version": a.data_type_version,
                "sampler": a.sampler,
                "data": serde_json::from_str::<J>(&a.data)?,
            }))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    Ok(Json(json!({
        "from": from,
        "to": to,
        "data": &v
    })))
}
#[get("/fetch-info/<from>/<to>")]
fn fetch_info(db: DbConn, from: String, to: String) -> anyhow::Result<Json<J>> {
    // println!("handling...");
    let from =
        DateTime::<chrono::FixedOffset>::parse_from_rfc3339(&from)?.with_timezone(&chrono::Utc);
    let to = DateTime::<chrono::FixedOffset>::parse_from_rfc3339(&to)?.with_timezone(&chrono::Utc);

    // println!("querying...");
    let mdata = {
        use trbtt::schema::activity::dsl::*;
        activity
            .filter(timestamp.ge(Timestamptz::new(from)))
            .filter(timestamp.lt(Timestamptz::new(to)))
            .load::<Activity>(&*db)?
    };
    // println!("jsonifying...");
    let v = mdata
        .into_iter()
        .filter_map(|a| {
            let r = deserialize_captured((&a.data_type, a.data_type_version, &a.data));
            match r {
                Ok(r) => {
                    if let Some(data) = extract_info(a.id.to_string(), &r) {
                        Some(json!({
                            "id": a.id,
                            "timestamp": a.timestamp,
                            "data_type": a.data_type,
                            "data_type_version": a.data_type_version,
                            "sampler": a.sampler,
                            "data": data,
                        }))
                    } else {
                        None
                    }
                }
                Err(e) => {
                    println!("deser error: {:?}", e);
                    None
                }
            }
        })
        .collect::<Vec<_>>();
    Ok(Json(json!({
        "from": from,
        "to": to,
        "data": &v
    })))
}

fn main() -> anyhow::Result<()> {
    let cors = rocket_cors::CorsOptions {
        allowed_origins: rocket_cors::AllowedOrigins::all(),
        ..Default::default()
    }
    .to_cors()?;
    rocket::ignite()
        .mount("/", routes![fetch_activity, fetch_info])
        .attach(cors)
        .attach(DbConn::fairing())
        .launch();

    Ok(())
}
