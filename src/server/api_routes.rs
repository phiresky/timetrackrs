use std::{future::ready, time::Duration};

use crate::api_types::*;
use crate::db::models::{DbEvent, Timestamptz};
use crate::prelude::*;
use crate::server::warp_util::{balanced_or_tree, debug_boxed};

use futures::StreamExt;
use warp::{reply::json, Filter, Rejection};

pub mod progress_events {
    use std::sync::Arc;

    use crate::prelude::*;

    use tokio::sync::broadcast::{Receiver, Sender};

    #[derive(Debug, Serialize, TypeScriptify)]
    pub struct ProgressReport {
        call_id: String,
        call_desc: String,
        state: Vec<ProgressState>,
        done: bool,
    }
    type SharedProgressReport = Arc<ProgressReport>;

    lazy_static::lazy_static! {
        static ref LOSSY_CHAN: (Sender<SharedProgressReport>, Receiver<SharedProgressReport>) = tokio::sync::broadcast::channel(1);
        static ref END_CHAN: (Sender<SharedProgressReport>, Receiver<SharedProgressReport>) = tokio::sync::broadcast::channel(100);
    }
    fn get_sender() -> (Sender<SharedProgressReport>, Sender<SharedProgressReport>) {
        (LOSSY_CHAN.0.clone(), END_CHAN.0.clone())
    }
    pub fn get_receiver() -> (
        Receiver<SharedProgressReport>,
        Receiver<SharedProgressReport>,
    ) {
        (LOSSY_CHAN.0.subscribe(), END_CHAN.0.subscribe())
    }

    #[derive(Debug)]
    struct StreamingReporter {
        call_id: String,
        call_desc: String,
        progress_sender: Sender<SharedProgressReport>,
        end_sender: Sender<SharedProgressReport>,
    }
    impl ProgressReporter for StreamingReporter {
        fn report(&self, state: Vec<ProgressState>) {
            let done = state.is_empty();
            let report = ProgressReport {
                call_id: self.call_id.clone(),
                call_desc: self.call_desc.clone(),
                state,
                done,
            };
            let sender = if report.done {
                &self.end_sender
            } else {
                &self.progress_sender
            };
            sender
                .send(Arc::new(report))
                .expect("Could not send progress");
        }
    }

    pub fn new_progress(desc: impl Into<String>) -> Progress {
        let id = crate::libxid::new_generator().new_id().unwrap().encode();
        println!("new id generated: {id}");
        let (progress_sender, end_sender) = get_sender();
        Progress::root(Arc::new(StreamingReporter {
            call_id: id,
            call_desc: desc.into(),
            progress_sender,
            end_sender,
        }))
    }
}

async fn get_known_tags(
    db: DatyBasy,
    _req: Api::get_known_tags::request,
) -> Api::get_known_tags::response {
    let tags = sqlx::query_scalar!("select text from extracted.tags")
        .fetch_all(&db.db)
        .await?;
    Ok(ApiResponse { data: tags })
}

async fn timestamp_search(
    db: DatyBasy,
    req: Api::timestamp_search::request,
) -> Api::timestamp_search::response {
    if req.backwards {
        let timestamp: Option<Option<Timestamptz>> = sqlx::query_scalar!(
            r#"select max(timestamp_unix_ms) as "e: Timestamptz" from raw_events.events where timestamp_unix_ms < ?"#,
            req.from
        )
        .fetch_optional(&db.db)
        .await?;
        Ok(ApiResponse {
            data: timestamp.flatten(),
        })
    } else {
        let timestamp: Option<Option<Timestamptz>> = sqlx::query_scalar!(
        r#"select min(timestamp_unix_ms) as "e: Timestamptz" from raw_events.events where timestamp_unix_ms > ?"#,
        req.from
    )
    .fetch_optional(&db.db)
    .await?;
        Ok(ApiResponse {
            data: timestamp.flatten(),
        })
    }
}

async fn time_range(db: DatyBasy, req: Api::time_range::request) -> Api::time_range::response {
    let progress = progress_events::new_progress("Extracting time range");

    let now = Instant::now();
    let data = db
        .get_extracted_for_time_range(req.after, req.before, req.tag.as_deref(), progress)
        .await
        .context("Could not get extracted events")?;

    log::debug!("time-range request took {:?}", now.elapsed());
    Ok(ApiResponse { data })
}

async fn invalidate_extractions(
    db: DatyBasy,
    req: Api::invalidate_extractions::request,
) -> Api::invalidate_extractions::response {
    db.invalidate_timechunks_range(req.from, req.to).await?;
    Ok(ApiResponse { data: () })
}

async fn single_events(
    db: DatyBasy,
    req: Api::single_events::request,
) -> Api::single_events::response {
    // println!("handling...");
    // println!("querying...");
    let ids_json = serde_json::to_string(&req.ids)?;
    let events: Vec<DbEvent> = sqlx::query_as!(
        DbEvent,
        r#"select insertion_sequence, id, timestamp_unix_ms as "timestamp_unix_ms: _",
    data_type, duration_ms, data from raw_events.events where id in (select value from json_each(?))"#,
        ids_json
    )
    .fetch_all(&db.db)
    .await?;

    let mut v: Vec<SingleExtractedEventWithRaw> = vec![];

    let progress = progress_events::new_progress("Single Events");

    for event in events {
        let r = event.deserialize_data();
        let progress = progress.clone();

        let ele = match r {
            Ok(raw) => {
                if let Some(data) = raw.extract_info() {
                    let (tags, tags_reasons) = {
                        if req.include_reasons {
                            let (tags, r, _) = get_tags_with_reasons(&db, data, progress).await;
                            (tags, Some(r))
                        } else {
                            let (tags, _) = get_tags(&db, data, progress).await;
                            (tags, None)
                        }
                    };
                    //let (tags, iterations) = get_tags(&db, data);
                    Some(SingleExtractedEventWithRaw {
                        id: event.id,
                        timestamp_unix_ms: event.timestamp_unix_ms,
                        duration_ms: event.duration_ms,
                        tags_reasons,
                        tags,
                        raw: req.include_raw.then_some(raw),
                    })
                } else {
                    None
                }
            }
            Err(e) => {
                println!("deser of {} error: {:?}", event.id, e);
                // println!("data=||{}", a.data);
                None
            }
        };
        if let Some(e) = ele {
            v.push(e);
        }
    }

    Ok(ApiResponse { data: v })
}

async fn rule_groups(db: DatyBasy) -> Api::rule_groups::response {
    Ok(ApiResponse {
        data: get_rule_groups(&db.db).await?.collect::<Vec<_>>(),
    })
}

async fn update_rule_groups(
    db: DatyBasy,
    req: Api::update_rule_groups::request,
) -> Api::update_rule_groups::response {
    // println!("handling...");
    // println!("querying...");

    for g in req {
        sqlx::query!(
            "insert into config.tag_rule_groups (global_id, data) values (?, ?)
                on conflict(global_id) do update set data = excluded.data",
            g.global_id,
            g.data
        )
        .execute(&db.db)
        .await?;
        let id = &g.global_id;
        log::debug!("updated rule group with id {id}")
    }
    db.reload_tag_rules()
        .await
        .context("Could not reload rule groups")?;

    Ok(ApiResponse { data: () })
}

#[derive(Debug)]
pub struct ErrAsJson {
    err: anyhow::Error,
}
impl warp::reject::Reject for ErrAsJson {}
impl ErrAsJson {
    pub fn to_json(&self) -> warp::reply::Json {
        warp::reply::json(&serde_json::json!({
            "message": format!("{:?}", &self.err)
        }))
    }
}

fn map_error(err: anyhow::Error) -> warp::Rejection {
    warp::reject::custom(ErrAsJson { err })
}

pub fn with_db(
    db: DatyBasy,
) -> impl warp::Filter<Extract = (DatyBasy,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}
pub fn api_routes(
    db: DatyBasy,
) -> impl warp::Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone + Send {
    let rule_groups = with_db(db.clone())
        .and(warp::path("rule-groups"))
        .and_then(|db| async { rule_groups(db).await.map(|e| json(&e)).map_err(map_error) });

    let time_range = with_db(db.clone())
        .and(warp::path("time-range"))
        .and(warp::query::<Api::time_range::request>())
        .and_then(|db, query| async {
            time_range(db, query)
                .await
                .map(|e| json(&e))
                .map_err(map_error)
        })
        .boxed();

    let timestamp_search = with_db(db.clone())
        .and(warp::path("timestamp-search"))
        .and(warp::query::<Api::timestamp_search::request>())
        .and_then(|db, query| async {
            timestamp_search(db, query)
                .await
                .map(|e| json(&e))
                .map_err(map_error)
        })
        .boxed();

    let get_known_tags = with_db(db.clone())
        .and(warp::path("get-known-tags"))
        .and(warp::query::<Api::get_known_tags::request>())
        .and_then(|db, query| async move {
            get_known_tags(db, query)
                .await
                .map(|e| json(&e))
                .map_err(map_error)
        })
        .boxed();
    let single_event = with_db(db.clone())
        .and(warp::path("single-events"))
        .and(warp::query::<Api::single_events::request>())
        .and_then(|db, query| async move {
            single_events(db, query)
                .await
                .map(|e| json(&e))
                .map_err(map_error)
        })
        .boxed();

    let update_rule_groups = warp::post()
        .and(with_db(db.clone()))
        .and(warp::path("rule-groups"))
        .and(warp::body::json())
        .and_then(|db, req| async move {
            update_rule_groups(db, req)
                .await
                .map(|e| json(&e))
                .map_err(map_error)
        })
        .boxed();
    let invalidate_extractions = warp::post()
        .and(with_db(db))
        .and(warp::path("invalidate-extractions"))
        .and(warp::query::<Api::invalidate_extractions::request>())
        .and_then(|db, query| async move {
            log::info!("OOOO");
            invalidate_extractions(db, query)
                .await
                .map(|e| json(&e))
                .map_err(map_error)
        })
        .boxed();

    let progress_events = warp::path("progress-events")
        .and(warp::get())
        .map(|| {
            let (lossy_progress_events, end_events) = progress_events::get_receiver();
            let end_events = tokio_stream::wrappers::BroadcastStream::new(end_events);
            let events = tokio_stream::wrappers::BroadcastStream::new(lossy_progress_events);

            // filter out and ignore the Lagged() err caused by polling behind a throttle
            let events = StreamExt::filter_map(events, |e| {
                ready(match e {
                    Ok(e) => Some(e),
                    Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(_)) => {
                        None
                    }
                })
            })
            .map(|e| vec![e]);
            // separate stream for end progress events that's not throttled
            let end_events = end_events
                .filter_map(|e| {
                    ready(match e {
                        Ok(e) => Some(e),
                        Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(
                            l,
                        )) => {
                            log::warn!("end progress event missed! {}", l);
                            None
                        }
                    })
                })
                .ready_chunks(10);
            let events = tokio_stream::StreamExt::throttle(events, Duration::from_millis(250));
            let merged = futures::stream::select(events, end_events)
                .map(|e| warp::sse::Event::default().json_data(e));
            warp::sse::reply(merged)
        })
        .boxed();

    let get_reqs = warp::get().and(balanced_or_tree!(
        time_range,
        get_known_tags,
        single_event,
        rule_groups,
        timestamp_search,
        progress_events
    ));
    let post_reqs = balanced_or_tree!(update_rule_groups, invalidate_extractions);

    get_reqs.or(post_reqs)
}
