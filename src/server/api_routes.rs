use std::pin::Pin;

use crate::db::models::{DbEvent, Timestamptz};
use crate::extract::ExtractInfo;
use crate::prelude::*;
use crate::util::iso_string_to_datetime;

use futures::Future;
use rust_embed::RustEmbed;
use warp::{
    reply::{json, Json},
    Filter, Rejection,
};

use crate::api_types::*;

async fn get_known_tags(db: DatyBasy) -> Api::get_known_tags::response {
    let tags = sqlx::query_scalar!("select text from extracted.tags")
        .fetch_all(&db.db)
        .await?;
    Ok(ApiResponse { data: tags })
}

async fn time_range(db: DatyBasy, req: Api::time_range::request) -> Api::time_range::response {
    // println!("handling...");
    // println!("querying...");
    let before = iso_string_to_datetime(&req.before).context("could not parse before date")?;
    let after = iso_string_to_datetime(&req.after).context("could not parse after date")?;

    Ok(ApiResponse {
        data: db
            .get_extracted_for_time_range(
                &Timestamptz(after),
                &Timestamptz(before),
                req.tag.as_deref(),
            )
            .await
            .context("get extracted events")?,
    })
}

async fn single_event(
    db: DatyBasy,
    req: Api::single_event::request,
) -> Api::single_event::response {
    // println!("handling...");
    // println!("querying...");
    let a = sqlx::query_as!(
        DbEvent,
        r#"select insertion_sequence, id, timestamp_unix_ms as "timestamp_unix_ms: _",
    data_type, duration_ms, data from raw_events.events where id = ?"#,
        req.id
    )
    .fetch_one(&db.db)
    .await?;

    let r = a.deserialize_data();
    let v = match r {
        Ok(raw) => {
            if let Some(data) = raw.extract_info() {
                let (tags, tags_reasons, _iterations) = get_tags_with_reasons(&db, data).await;
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

    Ok(ApiResponse { data: v })
}

async fn rule_groups(db: DatyBasy) -> Api::rule_groups::response {
    let groups = sqlx::query_as!(
        TagRuleGroup,
        r#"select global_id, data as "data: _" from config.tag_rule_groups"#
    )
    .fetch_all(&db.db)
    .await
    .context("fetching from db")?
    .into_iter()
    .chain(get_default_tag_rule_groups())
    .collect::<Vec<_>>();

    Ok(ApiResponse { data: groups })
}
#[derive(Debug)]
struct ErrAsJson {
    err: anyhow::Error,
}
impl warp::reject::Reject for ErrAsJson {}

fn map_error(err: anyhow::Error) -> warp::Rejection {
    return warp::reject::custom(ErrAsJson { err });
}

pub fn with_db(
    db: DatyBasy,
) -> impl warp::Filter<Extract = (DatyBasy,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}
fn time_range_route(
    db: DatyBasy,
    query: Api::time_range::request,
) -> Pin<Box<dyn Future<Output = Result<warp::reply::Json, Rejection>> + Send>> {
    Box::pin(async move {
        time_range(db, query)
            .await
            .map(|e| json(&e))
            .map_err(map_error)
    })
}
pub fn api_routes(
    db: DatyBasy,
) -> impl warp::Filter<Extract = (warp::reply::Json,), Error = Rejection> + Clone + Send {
    let rule_groups = with_db(db.clone())
        .and(warp::path("rule_groups"))
        .and_then(|db| async { rule_groups(db).await.map(|e| json(&e)).map_err(map_error) });

    let time_range = with_db(db.clone())
        .and(warp::path("time_range"))
        .and(warp::query::<Api::time_range::request>())
        .and_then(time_range_route);

    let filter = warp::get().and(rule_groups).or(time_range).unify();

    filter
}
/*
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
*/
