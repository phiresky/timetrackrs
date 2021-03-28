use std::{
    collections::{HashMap, HashSet},
    sync::{atomic::AtomicUsize, Arc},
    time::Instant,
};

use crate::{api_types::SingleExtractedEvent, prelude::*};
use futures::{future::join_all, stream::BoxStream, FutureExt};
use futures::{StreamExt, TryStreamExt};
use itertools::Itertools;
use sqlx::{Sqlite, SqlitePool};
use std::iter::FromIterator;
use std::sync::atomic::Ordering::Relaxed;
use tokio::sync::RwLock;

use super::caching_int_map::CachingIntMap;

pub async fn init_db_pool() -> anyhow::Result<DatyBasy> {
    let db = crate::db::connect(None)
        .await
        .context("Could not connect to db")?;
    Ok(DatyBasy {
        enabled_tag_rules: Arc::new(RwLock::new(
            fetch_tag_rules(&db).await.context("fetching tag rules")?,
        )),
        db: db.clone(),
        tags_cache: CachingIntMap::new(db.clone(), "tags", "(text) values (?1)", "text").await,
        values_cache: CachingIntMap::new(db.clone(), "tag_values", "(text) values (?1)", "text")
            .await,
        events_cache: CachingIntMap::new(
            db,
            "event_ids",
            "(raw_id, timestamp_unix_ms, duration_ms) values (?1, ?2, ?3)",
            "raw_id",
        )
        .await,
    })
}

#[derive(Clone)]
pub struct DatyBasy {
    pub db: SqlitePool,
    events_cache: CachingIntMap,
    tags_cache: CachingIntMap,
    values_cache: CachingIntMap,
    enabled_tag_rules: Arc<RwLock<Vec<TagRule>>>,
}

pub async fn fetch_tag_rules(db: &SqlitePool) -> anyhow::Result<Vec<TagRule>> {
    let groups: Vec<TagRuleGroup> = sqlx::query_as!(
        TagRuleGroup,
        r#"select global_id, data as "data: _" from config.tag_rule_groups"#
    )
    .fetch_all(db)
    .await?;
    /*if groups.len() == 0 {
        // insert defaults
        let groups =
        diesel::insert_into(tag_rule_groups)
            .values(groups)
            .execute(self.conn)?;
        return self.fetch_all_tag_rules_if_thoink();
    }*/

    Ok(groups
        .into_iter()
        .chain(get_default_tag_rule_groups().into_iter())
        .flat_map(|g| g.data.0.into_iter_active_rules())
        .collect())
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
        self.enabled_tag_rules.read().await
    }

    pub async fn get_extracted_for_time_range(
        &self,
        from: &Timestamptz,
        to: &Timestamptz,
        tag: Option<&str>,
        progress: Progress,
    ) -> anyhow::Result<Vec<SingleExtractedEvent>> {
        self.ensure_time_range_extracted_valid(
            from,
            to,
            progress.child(0, 1, "Ensuring extracted time range is valid"),
        )
        .await
        .context("Could not update extracted events")?;

        let now = Instant::now();
        let from = Timestamptz::from(from);
        let to = Timestamptz::from(to);
        let q = if let Some(tag) = tag {
            sqlx::query_as!(OutExtractedTag, r#"
            select e.timestamp_unix_ms as "timestamp: _", e.duration_ms, tags.text as tag, tag_values.text as value, event_ids.raw_id as event_id
            from extracted_events e
            join tags on tags.id = e.tag
            join tag_values on tag_values.id = e.value
            join event_ids on event_ids.id = e.event_id
            where e.tag = (select id from tags where text = ?3) and e.timestamp_unix_ms >= ?1 and e.timestamp_unix_ms < ?2
            order by e.timestamp_unix_ms desc"#, from, to, tag)
            .fetch_all(&self.db).await
            .context("querying extracted db")?
        } else {
            sqlx::query_as!(OutExtractedTag, r#"
            select e.timestamp_unix_ms as "timestamp: _", e.duration_ms, tags.text as tag, tag_values.text as value, event_ids.raw_id as event_id
            from extracted_events e
            join tags on tags.id = e.tag
            join tag_values on tag_values.id = e.value
            join event_ids on event_ids.id = e.event_id
            where e.timestamp_unix_ms >= ?1 and e.timestamp_unix_ms < ?2
            order by e.timestamp_unix_ms desc"#, from, to)
                .fetch_all(&self.db).await
                .context("querying extracted db")?
        };
        let ee = q.into_iter().group_by(|e| e.event_id.clone());
        let e: Vec<_> = ee
            .into_iter()
            .map(|(id, group)| {
                let mut group = group.peekable();
                SingleExtractedEvent {
                    id,
                    timestamp_unix_ms: (&group.peek().unwrap().timestamp).into(),
                    duration_ms: group.peek().unwrap().duration_ms,
                    tags: group.map(|e| (e.tag, e.value)).collect(),
                }
            })
            .collect();
        log::debug!("geting extracted from db took {:?}", now.elapsed());
        Ok(e)
    }
    pub async fn ensure_time_range_extracted_valid(
        &self,
        from: &Timestamptz,
        to: &Timestamptz,
        progress: Progress,
    ) -> anyhow::Result<()> {
        let days = self.get_affected_utc_days_range(from, to);
        {
            let days_str = serde_json::to_string(&days)?;
            let doesnt_need_update: Vec<DateUtc> = sqlx::query_scalar!( r#"select utc_date as "date_utc: _" from extracted.extracted_current where utc_date in (select value from json_each(?)) and extracted_timestamp_unix_ms > raw_events_changed_timestamp_unix_ms"#, days_str).fetch_all(&self.db).await.context("fetching currents")?;
            let doesnt_need_update = HashSet::<DateUtc>::from_iter(doesnt_need_update.into_iter());
            let needs_update: Vec<_> = days
                .into_iter()
                .filter(|e| !doesnt_need_update.contains(e))
                .collect();
            if needs_update.len() > 0 {
                let count = needs_update.len() as i64;
                progress.update(0, count, "Dates need update");
                let futures = needs_update
                    .into_iter()
                    .map(|day| {
                        let datybasy = self.clone();
                        let progress = progress.clone();
                        tokio::spawn(async move {
                            println!("started task {}", day.0);
                            datybasy.extract_time_range_and_store(&progress, day).await
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
        day: DateUtc,
    ) -> anyhow::Result<()> {
        let progress = progress.child_inc(format!("extracting day {}", day.0));
        let now = Timestamptz(Utc::now());
        self.extract_time_range(
            Timestamptz(day.0.and_hms(0, 0, 0)),
            Timestamptz((day.0 + chrono::Duration::days(1)).and_hms(0, 0, 0)),
            progress,
        )
        .await
        .with_context(|| format!("Could not extract tags for day {:?}", day))?;

        let updated = sqlx::query!("update extracted.extracted_current set extracted_timestamp_unix_ms = ? where utc_date = ?", now, day).execute(&self.db).await.with_context(|| format!("updating extracted timestamp {:?} {:?}", day, now))?.rows_affected();
        if updated == 0 {
            let zero = Timestamptz(Utc.ymd(1970, 1, 1).and_hms(0, 0, 0));
            sqlx::query!("insert into extracted.extracted_current (utc_date, extracted_timestamp_unix_ms, raw_events_changed_timestamp_unix_ms) values (?, ?, ?)", day, now, zero)
                        .execute(&self.db).await
                        .with_context(|| {
                            format!("inserting extracted timestamp {:?} {:?}", day, now)
                        })?;
        }
        Ok(())
    }
    fn get_affected_utc_days_range(&self, from: &Timestamptz, to: &Timestamptz) -> Vec<DateUtc> {
        let from_date = from.0.date();
        let to_date = to.0.date();
        let day = chrono::Duration::days(1);
        let mut date = from_date;
        let mut affected = Vec::new();
        while date <= to_date {
            affected.push(DateUtc(date));
            date = date + day;
        }
        affected
    }
    fn get_affected_utc_days_events<'a>(
        &self,
        events: impl IntoIterator<Item = &'a NewDbEvent>,
    ) -> HashSet<DateUtc> {
        let days: HashSet<DateUtc> = events
            .into_iter()
            .map(|e| DateUtc(e.timestamp_unix_ms.0.date()))
            .collect();
        days
    }
    async fn invalidate_extractions(&self, events: &[NewDbEvent]) -> anyhow::Result<()> {
        let days = self.get_affected_utc_days_events(events);
        let days_str = serde_json::to_string(&days).context("impossibo")?;
        let now = Timestamptz(Utc::now());
        sqlx::query!(
            r#"insert into extracted.extracted_current
                (utc_date, extracted_timestamp_unix_ms, raw_events_changed_timestamp_unix_ms)
                
            select json.value as utc_date, 0, ?
            from json_each(?) as json where true
            
            on conflict(utc_date) do update set raw_events_changed_timestamp_unix_ms = excluded.raw_events_changed_timestamp_unix_ms
            "#,
            now,
            days_str
        ).execute(&self.db).await.context("Could not update extracted_current")?;
        Ok(())
    }

    async fn extract_single_event(
        &self,
        a: DbEvent,
        r: Tags,
        progress: Progress,
        total_cache_get_dur: Arc<std::sync::RwLock<Duration>>,
    ) -> anyhow::Result<Vec<InExtractedTag>> {
        let aid = a.id.clone();

        //total_extracted += 1;
        //total_tags += r.tag_count();
        //total_tag_values += r.total_value_count();
        let timestamp = a.timestamp_unix_ms.clone();
        let duration_ms = a.duration_ms;
        let now = Instant::now();
        let event_id = self
            .events_cache
            .get_bind2(&aid, timestamp.0.timestamp_millis(), duration_ms)
            .await;
        *total_cache_get_dur.write().unwrap() += now.elapsed();
        let _now = Instant::now();
        let (tags, _iterations) = get_tags(&self, r, progress).await;
        //total_extract_dur += now.elapsed();
        //total_extract_iterations += iterations;

        Ok(futures::stream::iter(tags.into_iter())
            .flat_map(move |(tag, values)| {
                let timestamp = timestamp.clone();
                let t = total_cache_get_dur.clone();
                async move {
                    let now = Instant::now();
                    let tag = self.tags_cache.get(&tag).await;
                    *t.write().unwrap() += now.elapsed();

                    futures::stream::iter(values).then(move |value| {
                        let timestamp = timestamp.clone();
                        let t = t.clone();
                        async move {
                            let now = Instant::now();
                            let value = self.values_cache.get(&value).await;
                            *t.write().unwrap() += now.elapsed();
                            InExtractedTag {
                                timestamp_unix_ms: (&timestamp).into(),
                                duration_ms,
                                event_id,
                                tag,
                                value,
                            }
                        }
                    })
                }
                .flatten_stream()
            })
            .collect()
            .await)
    }

    // https://github.com/rust-lang/rust/issues/64552
    // https://github.com/rust-lang/rust/issues/64650
    pub async fn extract_time_range(
        &self,
        from: Timestamptz,
        to: Timestamptz,
        progress: Progress,
    ) -> anyhow::Result<()> {
        log::debug!("extract_time_range {:?} to {:?}", from, to);
        let now = Instant::now();
        progress.update(0, 3, "Removing stale events");
        {
            let res = sqlx::query!(
                "delete from extracted_events where timestamp_unix_ms >= ? and timestamp_unix_ms < ?",
                from, to)
            .execute(&self.db).await
            .context("Could not remove stale events")?;
            log::info!("removed {} stale events", res.rows_affected());
        }

        /*let raws = YieldEventsFromTrbttDatabase {
            db: &*self.db_events,
            chunk_size: 1000,
            last_fetched: from.clone(),
            ascending: true,
        };*/
        progress.update(1, 3, "Fetching raw events");
        let raws = sqlx::query_as!(
            DbEvent,
            r#"select
                insertion_sequence, id, timestamp_unix_ms as "timestamp_unix_ms: _",
                data_type, duration_ms, data
            from raw_events.events where timestamp_unix_ms >= ? and timestamp_unix_ms < ? order by timestamp_unix_ms asc"#,
            from, to
        )
        .fetch_all(&self.db).await?;
        log::info!(
            "Got {} raw events in range {} - {}",
            raws.len(),
            from.0,
            to.0
        );

        let now = Instant::now();
        let mut total_raw: usize = raws.len();
        let mut total_extracted = Arc::new(AtomicUsize::new(0));
        let mut total_tags: usize = 0;
        let mut total_tag_values: usize = 0;
        let mut total_extract_iterations: usize = 0;
        let mut total_extract_dur = Arc::new(std::sync::RwLock::new(Duration::from_secs(0)));
        let total_cache_get_dur = Arc::new(std::sync::RwLock::new(Duration::default()));

        let mut extracted: BoxStream<Vec<Result<_, _>>> = Box::pin(
            futures::stream::iter(raws.into_iter().filter_map(|a| {
                let r = a.deserialize_data();
                let ex: Tags = match r {
                    Ok(r) => r.extract_info()?,
                    Err(e) => {
                        log::warn!("{:#?}", e);
                        return None;
                    }
                };

                Some((a, ex))
            }))
            .then(|(a, r)| {
                let ts = a.timestamp_unix_ms.clone();
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
            })
            .chunks(1000),
        );

        while let Some(chunk) = extracted.next().await {
            let now = Instant::now();
            /*event_id bigint NOT NULL REFERENCES event_ids (id),
            timestamp_unix_ms bigint NOT NULL,
            duration_ms bigint NOT NULL,
            tag bigint NOT NULL REFERENCES tags (id),
            value*/
            let mut updated: usize = 0;
            let mut tx = self.db.begin().await?;
            for ele in chunk.into_iter().flatten().flatten() {
                sqlx::query!("insert into extracted.extracted_events (event_id, timestamp_unix_ms, duration_ms, tag, value) values (?, ?, ?, ?, ?)", ele.event_id, ele.timestamp_unix_ms, ele.duration_ms, ele.tag, ele.value)
                    .execute(&mut tx)
                    .await.context("inserting extracted events")?;
                updated += 1;
            }
            tx.commit().await?;
            log::info!("inserted {} ({:?})", updated, now.elapsed());
        }
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

        self.invalidate_extractions(&events)
            .await
            .context("Could not invalidate extractions")?;

        Ok(inserted)
    }
}
