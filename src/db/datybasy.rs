use std::{
    cmp::{max, min},
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicBool, AtomicUsize},
        Arc,
    },
    time::Instant,
};

use super::caching_int_map::CachingIntMap;
use crate::{api_types::SingleExtractedChunk, prelude::*};
use futures::StreamExt;
use futures::{stream::BoxStream, FutureExt};
use itertools::Itertools;
use sqlx::SqlitePool;
use std::iter::FromIterator;
use std::sync::atomic::Ordering::Relaxed;
use tokio::sync::RwLock;

pub async fn init_db_pool() -> anyhow::Result<DatyBasy> {
    static CALLED: AtomicBool = AtomicBool::new(false);
    CALLED.compare_exchange(false, true, Relaxed, Relaxed)
        .map_err(|_| anyhow::anyhow!("DB was already connected. Create new instances of DatyBasy by using .clone(), otherwise stuff like rules invalidation would not apply to all instances of DatyBasy"))?;
    let db = crate::db::connect(None)
        .await
        .context("Could not connect to db")?;
    Ok(DatyBasy {
        enabled_tag_rules: Arc::new(RwLock::new(Arc::new(
            fetch_tag_rules(&db).await.context("fetching tag rules")?,
        ))),
        db: db.clone(),
        tags_cache: CachingIntMap::new(db.clone(), "tags", "(text) values (?1)", "text").await,
        values_cache: CachingIntMap::new(db.clone(), "tag_values", "(text) values (?1)", "text")
            .await,
        /*events_cache: CachingIntMap::new(
            db,
            "event_ids",
            "(raw_id, timestamp_unix_ms, duration_ms) values (?1, ?2, ?3)",
            "raw_id",
        )
        .await,*/
    })
}

#[derive(Clone)]
pub struct DatyBasy {
    pub db: SqlitePool,
    // events_cache: CachingIntMap,
    tags_cache: CachingIntMap,
    values_cache: CachingIntMap,
    /// Arc<RwLock<Arc< should allow invalidating the tag rules for all clones of this datybasy
    /// by calling make_mut on the inner arc
    enabled_tag_rules: Arc<RwLock<Arc<Vec<TagRule>>>>,
}

pub async fn get_rule_groups(
    db: &SqlitePool,
) -> anyhow::Result<impl Iterator<Item = TagRuleGroup>> {
    let groups = sqlx::query_as!(
        TagRuleGroup,
        r#"select global_id, data as "data: _" from config.tag_rule_groups"#
    )
    .fetch_all(db)
    .await
    .context("fetching from db")?
    .into_iter()
    .chain(get_default_tag_rule_groups())
    .unique_by(|g| g.global_id.clone()); // TODO: don't allow overriding anything in default rule groups apart from enabled / disabled
    Ok(groups)
}

pub async fn fetch_tag_rules(db: &SqlitePool) -> anyhow::Result<Vec<TagRule>> {
    Ok(get_rule_groups(db)
        .await?
        .flat_map(|g| g.data.0.into_iter_active_rules())
        .collect())
}

struct EventWithTagMap {
    timestamp: Timestamptz,
    duration_ms: i64,
    tags: Vec<(i64, i64)>,
}
struct SingleExtractedChunkInfo {
    timechunk: TimeChunk,
    tag: String,
    value: String,
    duration_ms: i64,
}
struct ExtractedChunks {
    // chunk -> (tag, value) -> duration_ms
    data: HashMap<TimeChunk, HashMap<(i64, i64), i64>>,
}
impl ExtractedChunks {
    fn new() -> ExtractedChunks {
        ExtractedChunks {
            data: HashMap::new(),
        }
    }
    fn add(&mut self, event: EventWithTagMap) {
        for (chunk, duration_ms) in get_affected_timechunks_duration_ms(
            event.timestamp,
            Timestamptz(event.timestamp.0 + chrono::Duration::milliseconds(event.duration_ms)),
        ) {
            let hm = self.data.entry(chunk).or_insert(HashMap::new());
            for tag in &event.tags {
                *hm.entry(*tag).or_insert(0) += duration_ms;
            }
        }
    }
    fn into_data(self) -> HashMap<TimeChunk, HashMap<(i64, i64), i64>> {
        self.data
    }
}

fn get_affected_timechunks_duration_ms(
    from: Timestamptz,
    to: Timestamptz,
) -> Vec<(TimeChunk, i64)> {
    let from_date = TimeChunk::containing(from.0).start();
    let interval = chrono::Duration::minutes(CHUNK_LEN_MINS as i64);
    let mut timechunk_start = from_date;
    let mut affected = Vec::new();
    while timechunk_start <= to.0 {
        let timechunk_end = timechunk_start + interval;
        let chunk = TimeChunk::at(timechunk_start)
            .with_context(|| format!("chunk at {timechunk_start:?}"))
            .unwrap();
        let duration =
            to.0.min(timechunk_end)
                .signed_duration_since(from.0.max(timechunk_start));
        affected.push((chunk, duration.num_milliseconds()));
        timechunk_start = timechunk_end;
    }
    affected
}

impl DatyBasy {
    pub async fn get_fetcher_cache_entry(
        &self,
        cache_key: &str,
    ) -> anyhow::Result<Option<FetchResultJson>> {
        let cache_value = sqlx::query_scalar!(
            "select value from extracted.fetcher_cache where key = ?",
            cache_key
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(if let Some(cache_value) = cache_value {
            Some(serde_json::from_str(&cache_value).context("deserializing cache")?)
        } else {
            None
        })
    }

    pub async fn set_fetcher_cache_entry(
        &self,
        cache_key: &str,
        cache_value: &FetchResultJson,
    ) -> anyhow::Result<()> {
        let now = Timestamptz(Utc::now());
        let cache_value = serde_json::to_string(&cache_value).context("serializing cache")?;
        sqlx::query!(
            "insert or replace into extracted.fetcher_cache (key, timestamp_unix_ms, value) values (?, ?, ?)",
            cache_key,
            now,
            cache_value
        )
        .execute(&self.db)
        .await
        .context("insert into fetcher_cache db")?;

        Ok(())
    }

    pub async fn get_all_tag_rules<'a>(
        &'a self,
    ) -> impl core::ops::Deref<Target = Vec<TagRule>> + 'a {
        // return a clone of the inner arc, so when make_mut is called on the inner arc the returned thing works undisturbed
        // and the read lock does not stay for the whole duration of the tag processing
        self.enabled_tag_rules.read().await.clone()
    }

    pub async fn reload_tag_rules<'a>(&'a self) -> anyhow::Result<()> {
        let mut writer = self.enabled_tag_rules.write().await;
        let writer = Arc::make_mut(&mut writer);
        *writer = fetch_tag_rules(&self.db)
            .await
            .context("fetching tag rules")?;
        Ok(())
    }

    pub async fn get_extracted_for_time_range(
        &self,
        from: Timestamptz,
        to: Timestamptz,
        tag: Option<&str>,
        progress: Progress,
    ) -> anyhow::Result<Vec<SingleExtractedChunk>> {
        self.ensure_time_range_extracted_valid(
            from,
            to,
            progress.child(0, 1, "Ensuring extracted time range is valid"),
        )
        .await
        .context("Could not update extracted events")?;

        let now = Instant::now();
        let from = TimeChunk::containing(from.0);
        let to = TimeChunk::containing(to.0);
        let q = if let Some(tag) = tag {
            sqlx::query_as!(SingleExtractedChunkInfo, r#"
            select e.timechunk as "timechunk: _", e.duration_ms, tags.text as tag, tag_values.text as value
            from extracted_chunks e
            join tags on tags.id = e.tag
            join tag_values on tag_values.id = e.value
            where e.tag = (select id from tags where text = ?3)
            and e.timechunk >= ?1 and e.timechunk <= ?2
            order by e.timechunk desc"#, from, to, tag)
            .fetch_all(&self.db).await
            .context("querying extracted db")?
        } else {
            sqlx::query_as!(SingleExtractedChunkInfo, r#"
            select e.timechunk as "timechunk: _", e.duration_ms, tags.text as tag, tag_values.text as value
            from extracted_chunks e
            join tags on tags.id = e.tag
            join tag_values on tag_values.id = e.value
            where e.timechunk >= ?1 and e.timechunk <= ?2
            order by e.timechunk desc"#, from, to)
            .fetch_all(&self.db).await
            .context("querying extracted db")?
        };
        let ee = q.into_iter().group_by(|e| e.timechunk);
        let e: Vec<_> = ee
            .into_iter()
            .map(|(id, group)| {
                let group = group.peekable();
                let timechunk = id;
                SingleExtractedChunk {
                    from: Timestamptz(timechunk.start()),
                    to_exclusive: Timestamptz(timechunk.end_exclusive()),
                    tags: group.map(|e| (e.tag, e.value, e.duration_ms)).collect(),
                }
            })
            .collect();
        log::debug!("geting extracted from db took {:?}", now.elapsed());
        Ok(e)
    }
    pub async fn ensure_time_range_extracted_valid(
        &self,
        from: Timestamptz,
        to: Timestamptz,
        progress: Progress,
    ) -> anyhow::Result<()> {
        let chunks = self.get_affected_timechunks_range(from, to);
        {
            let days_str = serde_json::to_string(&chunks)?;
            let doesnt_need_update: Vec<TimeChunk> = sqlx::query_scalar!(
                r#"
                select timechunk as "timechunk: _"
                from extracted.extracted_current
                where timechunk in (select value from json_each(?))
                and extracted_timestamp_unix_ms > raw_events_changed_timestamp_unix_ms"#,
                days_str
            )
            .fetch_all(&self.db)
            .await
            .context("fetching currents")?;
            let doesnt_need_update =
                HashSet::<TimeChunk>::from_iter(doesnt_need_update.into_iter());
            let mut needs_update: Vec<_> = chunks
                .into_iter()
                .filter(|e| !doesnt_need_update.contains(e))
                .collect();
            if !needs_update.is_empty() {
                log::debug!("chunks that need update: {:?}", needs_update);
                needs_update.sort();

                let mut all_out = vec![];
                let mut current_out = (needs_update[0], needs_update[0]);
                for ele in needs_update {
                    let distance_to_start =
                        ele.start().signed_duration_since(current_out.0.start());
                    let distance_to_end = ele.start().signed_duration_since(current_out.1.start());
                    // max 1 day
                    if distance_to_start >= chrono::Duration::days(1)
                        || distance_to_end > chrono::Duration::minutes(10)
                    {
                        all_out.push(current_out);
                        current_out = (ele, ele);
                    }
                    current_out.1 = ele;
                }
                all_out.push(current_out);
                log::debug!("Aggregated ranges: {:?}", all_out);
                let count = all_out.len() as i64;
                progress.update(0, count, "Ranges need update");
                let futures = all_out
                    .into_iter()
                    .map(|(start, end)| {
                        let datybasy = self.clone();
                        let progress = progress.clone();
                        tokio::spawn(async move {
                            datybasy
                                .extract_time_range_and_store(
                                    &progress,
                                    Timestamptz(start.start()),
                                    Timestamptz(end.end_exclusive()),
                                )
                                .await
                        })
                    })
                    .collect::<Vec<_>>();

                for future in futures {
                    future.await??;
                }
                /*for (i, day) in needs_update.into_iter().enumerate() {
                    self.extract_time_range_and_store(&progress, day).await?;
                }*/
            }
            Ok(())
        }
    }
    async fn extract_time_range_and_store(
        &self,
        progress: &Progress,
        start: Timestamptz,
        end: Timestamptz,
    ) -> anyhow::Result<()> {
        let progress = progress.child_inc(format!("extracting {start:?} - {end:?}"));
        self.extract_time_range(start, end, progress)
            .await
            .with_context(|| {
                format!(
                    "Could not extract tags for {:?} - {:?}",
                    start.clone(),
                    end.clone()
                )
            })?;

        self.mark_extractions_valid(start, end).await?;
        Ok(())
    }
    fn get_affected_timechunks_range(&self, from: Timestamptz, to: Timestamptz) -> Vec<TimeChunk> {
        let from_date = TimeChunk::containing(from.0).start();
        let to_date = to.0;
        let interval = chrono::Duration::minutes(CHUNK_LEN_MINS as i64);
        let mut date = from_date;
        let mut affected = Vec::new();
        while date <= to_date {
            affected.push(
                TimeChunk::at(date)
                    .with_context(|| format!("chunk at {date:?}"))
                    .unwrap(),
            );
            date = date + interval;
        }
        affected
    }
    fn get_affected_timechunks_events<'a>(
        &self,
        events: impl IntoIterator<Item = &'a NewDbEvent>,
    ) -> HashSet<TimeChunk> {
        let days: HashSet<TimeChunk> = events
            .into_iter()
            .map(|e| TimeChunk::containing(e.timestamp_unix_ms.0))
            .collect();
        days
    }
    pub async fn invalidate_timechunks_events(&self, events: &[NewDbEvent]) -> anyhow::Result<()> {
        let chunks = self.get_affected_timechunks_events(events);
        self.invalidate_timechunks(&chunks).await
    }
    pub async fn invalidate_timechunks_range(
        &self,
        from: Timestamptz,
        to: Timestamptz,
    ) -> anyhow::Result<()> {
        let existing_range = sqlx::query!(
            r#"
            select min(timechunk) as "first: TimeChunk", max(timechunk) as "last: TimeChunk"
            from extracted.extracted_current"#,
        )
        .fetch_one(&self.db)
        .await
        .context("fetching currents")?;
        if existing_range.first.is_none() {
            return Ok(());
        }
        let from = max(
            from,
            existing_range
                .first
                .map(|r| Timestamptz(r.start()))
                .unwrap(),
        );
        let to = min(
            to,
            existing_range.last.map(|r| Timestamptz(r.start())).unwrap(),
        );
        let chunks = self.get_affected_timechunks_range(from, to);
        log::debug!("Invalidating {from:?} to {to:?}");
        self.invalidate_timechunks(&chunks).await
    }
    // TODO: accept HashSet<TimeChunk> and Vec<TimeChunk> only
    async fn invalidate_timechunks(&self, chunks: impl Serialize) -> anyhow::Result<()> {
        let chunks_str = serde_json::to_string(&chunks).context("impossibo")?;
        let now = Timestamptz(Utc::now());
        sqlx::query!(
            r#"insert into extracted.extracted_current
                (timechunk, extracted_timestamp_unix_ms, raw_events_changed_timestamp_unix_ms)
                
            select json.value as timechunk, 0, ?
            from json_each(?) as json where true
            
            on conflict(timechunk) do update set raw_events_changed_timestamp_unix_ms = excluded.raw_events_changed_timestamp_unix_ms
            "#,
            now,
            chunks_str
        ).execute(&self.db).await.context("Could not update extracted_current")?;
        Ok(())
    }
    async fn mark_extractions_valid(
        &self,
        from: Timestamptz,
        to: Timestamptz,
    ) -> anyhow::Result<()> {
        let chunks = self.get_affected_timechunks_range(from, to);
        let chunks_str = serde_json::to_string(&chunks).context("impossibo")?;
        let now = Timestamptz(Utc::now());
        sqlx::query!(
            r#"insert into extracted.extracted_current
                (timechunk, extracted_timestamp_unix_ms, raw_events_changed_timestamp_unix_ms)
                
            select json.value as timechunk, ?, 0
            from json_each(?) as json where true
            
            on conflict(timechunk) do update set extracted_timestamp_unix_ms = excluded.extracted_timestamp_unix_ms
            "#,
            now,
            chunks_str
        ).execute(&self.db).await.context("Could not update extracted_current")?;
        Ok(())
    }

    async fn extract_single_event(
        &self,
        a: DbEvent,
        r: Tags,
        progress: Progress,
        total_cache_get_dur: Arc<std::sync::RwLock<Duration>>,
    ) -> anyhow::Result<EventWithTagMap> {
        //total_extracted += 1;
        //total_tags += r.tag_count();
        //total_tag_values += r.total_value_count();
        let timestamp = a.timestamp_unix_ms;
        let duration_ms = a.duration_ms;
        let now = Instant::now();
        *total_cache_get_dur.write().unwrap() += now.elapsed();
        let _now = Instant::now();
        let (tags, _iterations) = get_tags(self, r, progress).await;
        //total_extract_dur += now.elapsed();
        //total_extract_iterations += iterations;

        let tags: Vec<(i64, i64)> = futures::stream::iter(tags.into_iter())
            .flat_map(move |(tag, values)| {
                let t = total_cache_get_dur.clone();
                async move {
                    let now = Instant::now();
                    let tag = self.tags_cache.get(&tag).await;
                    *t.write().unwrap() += now.elapsed();

                    futures::stream::iter(values).then(move |value| {
                        let t = t.clone();
                        async move {
                            let now = Instant::now();
                            let value = self.values_cache.get(&value).await;
                            *t.write().unwrap() += now.elapsed();
                            (tag, value)
                        }
                    })
                }
                .flatten_stream()
            })
            .collect()
            .await;
        Ok(EventWithTagMap {
            timestamp,
            duration_ms,
            tags,
        })
    }

    // https://github.com/rust-lang/rust/issues/64552
    // https://github.com/rust-lang/rust/issues/64650
    // from and to must be timechunk-aligned
    pub async fn extract_time_range(
        &self,
        from: Timestamptz,
        to: Timestamptz,
        progress: Progress,
    ) -> anyhow::Result<()> {
        let _now = Instant::now();

        /*let raws = YieldEventsFromTrbttDatabase {
            db: &*self.db_events,
            chunk_size: 1000,
            last_fetched: from.clone(),
            ascending: true,
        };*/
        progress.update(1, 3, "Fetching raw events");
        let absolute_lower_bound =
            Timestamptz(from.0 - chrono::Duration::seconds(MAX_EVENT_LEN_SECS));
        let raws = sqlx::query_as!(
            DbEvent,
            r#"select
                insertion_sequence, id, timestamp_unix_ms as "timestamp_unix_ms: _",
                data_type, duration_ms, data
            from raw_events.events where
            timestamp_unix_ms + duration_ms >= ? and timestamp_unix_ms < ?
                and timestamp_unix_ms >= ? 
            order by timestamp_unix_ms asc"#,
            from,
            to,
            absolute_lower_bound // needed for perf
        )
        .fetch_all(&self.db)
        .await?;
        log::debug!(
            "Got {} raw events in range {} - {}",
            raws.len(),
            from.0,
            to.0
        );

        let _now = Instant::now();
        let total_raw: usize = raws.len();
        let total_extracted = Arc::new(AtomicUsize::new(0));
        let total_tags: usize = 0;
        let total_tag_values: usize = 0;
        let total_extract_iterations: usize = 0;
        let total_extract_dur = Arc::new(std::sync::RwLock::new(Duration::from_secs(0)));
        let total_cache_get_dur = Arc::new(std::sync::RwLock::new(Duration::default()));

        let mut extracted_chunks = ExtractedChunks::new();

        let mut extracted: BoxStream<Result<_, _>> = Box::pin(
            futures::stream::iter(raws.into_iter().filter_map(|a| {
                let r = a.deserialize_data();
                let ex: Tags = match r {
                    Ok(r) => {
                        let mut tags = r.extract_info()?;
                        tags.add("timetrackrs-tracked", "true");
                        tags.add("timetrackrs-data-source", &a.data_type);
                        tags.add("timetrackrs-raw-id", &a.id);
                        tags
                    }
                    Err(e) => {
                        log::warn!("{:#?}", e);
                        return None;
                    }
                };
                Some((a, ex))
            }))
            .then(|(a, r)| {
                let ts = a.timestamp_unix_ms;
                let p = progress.child(2, 3, format!("Extracting data for event {}", ts.0));
                let to = total_extract_dur.clone();
                let t2 = total_cache_get_dur.clone();
                total_extracted.fetch_add(1, Relaxed);
                async move {
                    let now = Instant::now();
                    let res = self.extract_single_event(a, r, p, t2).await;
                    *to.write().unwrap() += now.elapsed();
                    res
                }
            }),
        );
        while let Some(event) = extracted.next().await {
            extracted_chunks.add(event?);
        }

        let mut tx = self.db.begin().await?;
        let mut updated: usize = 0;
        let mut now = Instant::now();
        for (timechunk, events) in extracted_chunks.into_data() {
            log::debug!("looking at {} (from={})", timechunk.start(), from.0);
            if timechunk.start() < from.0 || timechunk.end_exclusive() > to.0 {
                log::debug!("skipping!");
                // we only requested data for the range `from` to `to`, so the data we received for
                // this chunk is incomplete / invalid
                continue;
            }
            /*event_id bigint NOT NULL REFERENCES event_ids (id),
            timestamp_unix_ms bigint NOT NULL,
            duration_ms bigint NOT NULL,
            tag bigint NOT NULL REFERENCES tags (id),
            value*/

            sqlx::query!(
                "delete from extracted.extracted_chunks where timechunk = ?",
                timechunk
            )
            .execute(&mut tx)
            .await
            .context("Could not remove stale events")?;

            for ((tag, value), duration_ms) in events.into_iter() {
                sqlx::query!("insert into extracted.extracted_chunks (timechunk, tag, value, duration_ms) values (?, ?, ?, ?)", timechunk, tag, value, duration_ms)
                    .execute(&mut tx)
                    .await.context("inserting extracted events")?;
                updated += 1;
            }
            if updated > 3000 {
                log::info!("inserted {} ({:?})", updated, now.elapsed());
                now = Instant::now();
                tx.commit().await?;
                updated = 0;
                tx = self.db.begin().await?;
            }
        }
        tx.commit().await?;

        let total_extracted = total_extracted.load(Relaxed);
        if total_extracted > 0 && total_raw > 0 {
            log::debug!(
                "extraction yielded {} extracted of {} raw events with {} tags with {} values total. extracting tags took {:?} total, avg. {} it/ev, avg. {:?} per ele, extracting avg. {:?} per ele, cachget avg. {:?} per ele",
                total_extracted,
                total_raw,
                total_tags,
                total_tag_values,
                now.elapsed(),
                 total_extract_iterations / total_extracted,
                now.elapsed().div_f32(total_raw as f32),
                total_extract_dur.read().unwrap().div_f32(total_raw as f32),
                total_cache_get_dur.read().unwrap().div_f32(total_raw as f32)
            );
            // log::debug!("cache stats")
        }
        log::info!(
            "extract_time_range {:?} to {:?} took {:?}",
            from,
            to,
            now.elapsed()
        );
        Ok(())
    }

    pub async fn insert_events_if_needed(&self, events: Vec<NewDbEvent>) -> anyhow::Result<u64> {
        let mut inserted: u64 = 0;

        let mut db = self.db.begin().await?;
        for event in &events {
            let res = sqlx::query!("insert or ignore into raw_events.events (id, timestamp_unix_ms, data_type, duration_ms, data) values (?, ?, ?, ?, ?)",
            event.id, event.timestamp_unix_ms, event.data_type, event.duration_ms, event.data).execute(&mut db).await.context("could not insert event")?;
            inserted += res.rows_affected();
        }
        db.commit().await?;

        // TODO: filter invalidation by RETURNING timestamp_unix_ms above
        self.invalidate_timechunks_events(&events)
            .await
            .context("Could not invalidate extractions")?;

        Ok(inserted)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_get_affected_timechunks_duration_ms() {
        let from = Timestamptz(Utc.timestamp(1620585292, 562 * 1_000_000));
        let res = get_affected_timechunks_duration_ms(
            from,
            Timestamptz(from.0 + chrono::Duration::milliseconds(30000)),
        );
        println!("resulting chunks: {res:?}");
    }
}
