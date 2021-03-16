use rocket::{
    get,
    http::{ContentType, Status},
    post, response, routes,
};
use rocket_contrib::json::Json;

use crate::db::models::{DbEvent, Timestamptz};
use crate::extract::ExtractInfo;
use crate::prelude::*;
use crate::util::iso_string_to_datetime;

use rust_embed::RustEmbed;

use crate::api_types::*;

#[derive(RustEmbed)]
#[folder = "frontend/dist/"]
struct FrontendDistAssets;
use crate as trbtt;

#[get("/get-known-tags")]
fn get_known_tags(db: DatyBasy) -> Api::get_known_tags::response {
    use trbtt::db::schema::extracted::tags::dsl::*;

    let all_tags = tags
        .select(text)
        .load(&*db.db_extracted)
        .context("loading tag names from db")?;

    Ok(Json(ApiResponse { data: all_tags }))
}

#[get("/time-range?<after>&<before>&<tag>")]
fn time_range(
    db: DatyBasy,
    before: String,
    after: String,
    tag: Option<String>,
) -> Api::time_range::response {
    // println!("handling...");
    // println!("querying...");
    let before = iso_string_to_datetime(&before).context("could not parse before date")?;
    let after = iso_string_to_datetime(&after).context("could not parse after date")?;

    Ok(Json(ApiResponse {
        data: db
            .get_extracted_for_time_range(&Timestamptz(after), &Timestamptz(before), tag.as_deref())
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
                let (tags, tags_reasons, _iterations) = get_tags_with_reasons(&db, data);
                //let (tags, iterations) = get_tags(&db, data);
                Some(SingleExtractedEventWithRaw {
                    id: a.id,
                    timestamp_unix_ms: a.timestamp_unix_ms,
                    duration_ms: a.duration_ms,
                    tags_reasons,
                    tags,
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
