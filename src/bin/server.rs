#![feature(proc_macro_hygiene, decl_macro)]

use diesel::prelude::*;
use rocket::{get, routes};
use rocket_contrib::json::Json;
use serde_json::json;
use serde_json::Value as J;
use track_pc_usage_rs as trbtt;
use track_pc_usage_rs::capture::deserialize_captured;
use track_pc_usage_rs::util::iso_string_to_date;
use trbtt::extract::ExtractInfo;
use trbtt::models::{Activity, Timestamptz};
#[macro_use]
extern crate rocket_contrib;

#[database("activity_database")]
struct DbConn(diesel::SqliteConnection);

/*#[get("/fetch-activity?<from>&<to>&<limit>")]
fn fetch_activity(
    db: DbConn,
    from: Option<String>,
    limit: Option<u32>,
    to: String,
) -> anyhow::Result<Json<J>> {
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
    Ok(Json(json!({ "data": &v })))
}*/
#[get("/fetch-info?<after>&<before>&<limit>")]
fn fetch_info(
    db: DbConn,
    after: Option<String>,
    limit: Option<u32>,
    before: Option<String>,
) -> anyhow::Result<Json<J>> {
    // println!("handling...");
    // println!("querying...");
    let mdata = {
        use trbtt::schema::activity::dsl::*;
        let mut query = activity.into_boxed();
        // let query = activity.filter(timestamp.lt(Timestamptz::new(to)));
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
        query.limit(limit as i64).load::<Activity>(&*db)?
    };
    // println!("jsonifying...");
    let v = mdata
        .into_iter()
        .filter_map(|a| {
            let r = deserialize_captured((&a.data_type, a.data_type_version, &a.data));
            match r {
                Ok(r) => {
                    if let Some(data) = r.extract_info(a.id.to_string()) {
                        Some(json!({
                            "id": a.id,
                            "timestamp": a.timestamp,
                            "duration": a.sampler.get_duration(),
                            "data": data,
                        }))
                    } else {
                        None
                    }
                }
                Err(e) => {
                    println!("deser of {} error: {:?}", a.id, e);
                    None
                }
            }
        })
        .collect::<Vec<_>>();
    Ok(Json(json!({ "data": &v })))
}

fn main() -> anyhow::Result<()> {
    let cors = rocket_cors::CorsOptions {
        allowed_origins: rocket_cors::AllowedOrigins::all(),
        ..Default::default()
    }
    .to_cors()?;
    rocket::ignite()
        .mount("/", routes![fetch_info])
        .attach(cors)
        .attach(DbConn::fairing())
        .launch();

    Ok(())
}
