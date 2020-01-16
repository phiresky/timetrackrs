#![feature(proc_macro_hygiene, decl_macro)]

use chrono::{DateTime, Local};
use diesel::prelude::*;
use rocket::{get, routes};
use rocket_contrib::json::Json;
use serde_json::json;
use serde_json::Value as J;
use track_pc_usage_rs as trbtt;
use trbtt::models::{Activity, Timestamptz};

#[get("/fetch-activity/<from>/<to>")]
fn hello(from: String, to: String) -> anyhow::Result<Json<J>> {
    let db = trbtt::database::connect()?;
    let from =
        DateTime::<chrono::FixedOffset>::parse_from_rfc3339(&from)?.with_timezone(&chrono::Utc);
    let to = DateTime::<chrono::FixedOffset>::parse_from_rfc3339(&to)?.with_timezone(&chrono::Utc);

    use trbtt::schema::activity::dsl::*;
    let mdata = activity
        .filter(timestamp.ge(Timestamptz::new(from)))
        .filter(timestamp.lt(Timestamptz::new(to)))
        .load::<Activity>(&db)?;

    let v = mdata
        .into_iter()
        .map(|a| {
            Ok(json!({
                "timestamp": a.timestamp,
                "data_type": a.data_type,
                "data_type_version": a.data_type_version,
                "sampler": serde_json::from_str::<J>(&a.sampler)?,
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

fn main() {
    rocket::ignite().mount("/", routes![hello]).launch();
}
