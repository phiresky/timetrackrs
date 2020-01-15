#![feature(proc_macro_hygiene, decl_macro)]

use chrono::{DateTime, Local};
use diesel::prelude::*;
use rocket::{get, routes};
use track_pc_usage_rs as trbtt;
use trbtt::models::{Activity, Timestamptz};

#[get("/fetch-activity/<from>/<to>")]
fn hello(from: String, to: String) -> anyhow::Result<String> {
    let db = trbtt::database::connect()?;
    let from =
        DateTime::<chrono::FixedOffset>::parse_from_rfc3339(&from)?.with_timezone(&chrono::Utc);
    let to = DateTime::<chrono::FixedOffset>::parse_from_rfc3339(&to)?.with_timezone(&chrono::Utc);

    use trbtt::schema::activity::dsl::*;
    let mdata = activity
        .filter(timestamp.gt(Timestamptz::new(from)))
        .filter(timestamp.lt(Timestamptz::new(to)))
        .load::<Activity>(&db)?;
    Ok(format!("Hello, {}", serde_json::to_string_pretty(&mdata)?))
}

fn main() {
    rocket::ignite().mount("/", routes![hello]).launch();
}
