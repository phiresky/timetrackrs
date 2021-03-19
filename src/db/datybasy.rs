use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    pin::Pin,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{api_types::SingleExtractedEvent, prelude::*};
use futures::{future::BoxFuture, stream::BoxStream, Future, FutureExt};
use futures::{StreamExt, TryStreamExt};
use itertools::Itertools;
use sqlx::{query::Query, sqlite::SqliteArguments, Sqlite, SqlitePool};
use std::iter::FromIterator;
use tokio::sync::RwLock;

/*
https://stackoverflow.com/questions/41665345/borrow-problems-with-compiled-sql-statements
https://stackoverflow.com/questions/32209391/how-to-store-rusqlite-connection-and-statement-objects-in-the-same-struct-in-rus
https://stackoverflow.com/questions/27552670/how-to-store-sqlite-prepared-statements-for-later
*/
#[derive(Clone)]
struct CachingIntMap {
    lru: Arc<RwLock<HashMap<String, i64>>>,
    pub conn: SqlitePool,
    get: String,
    put: String,
}

/*fn events_cache_insert(
    conn: SqlitePool,
    key: String,
    inp: (i64, i64),
) -> Pin<Box<dyn Future<Output = i64>>> {
    Box::pin(async {
        let ret = sqlx::query!("insert into extracted.event_ids (raw_id, timestamp_unix_ms, duration_ms) values (?1, ?2, ?3)", key, inp.0, inp.1)
        .execute(&conn)
        .await
        .unwrap();
        ret.last_insert_rowid()
    })
}*/

pub async fn init_db_pool() -> anyhow::Result<DatyBasy> {
    let db = crate::db::connect().await.context("connecting to db")?;
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

trait MyBonk<'a>: sqlx::Encode<'a, Sqlite> + sqlx::Type<Sqlite> {}

impl CachingIntMap {
    async fn new(conn: SqlitePool, table: &str, cols: &str, keycol: &str) -> CachingIntMap {
        lazy_static! {
            static ref lrus: Arc<RwLock<HashMap<String, Arc<RwLock<HashMap<String, i64>>>>>> =
                Arc::new(RwLock::new(HashMap::new()));
        }
        CachingIntMap {
            lru: (*lrus)
                .write()
                .await
                .entry(table.to_string())
                .or_insert_with(|| Arc::new(RwLock::new(HashMap::with_capacity(10_000))))
                .clone(),
            get: format!("select id from {} where {} = ?1", table, keycol),
            put: format!("insert into {} {}", table, cols),
            conn,
        }
    }
    async fn get(&self, key: &str) -> i64 {
        let i: Option<i64> = self.lru.write().await.get(key).copied();

        match i {
            Some(i) => i,
            None => {
                let n = match sqlx::query_scalar(&self.get)
                    .bind(key)
                    .fetch_optional(&self.conn)
                    .await
                    .unwrap()
                {
                    Some(n) => n,
                    None => {
                        let q = sqlx::query(&self.put).bind(key);
                        let ret = q.execute(&self.conn).await.unwrap();
                        ret.last_insert_rowid()
                    }
                };

                self.lru.write().await.insert(key.to_string(), n);
                n
            }
        }
    }
    // very shitty code. spent 2 hours figuring out how to do this get method generically
    async fn get_bind2(&'c self, key: &'c str, bind1: i64, bind2: i64) -> i64 {
        let i: Option<i64> = self.lru.write().await.get(key).copied();

        match i {
            Some(i) => i,
            None => {
                let n = match sqlx::query_scalar(&self.get)
                    .bind(key)
                    .fetch_optional(&self.conn)
                    .await
                    .unwrap()
                {
                    Some(n) => n,
                    None => {
                        let q = sqlx::query(&self.put).bind(key).bind(bind1).bind(bind2);
                        let ret = q.execute(&self.conn).await.unwrap();
                        ret.last_insert_rowid()
                    }
                };

                self.lru.write().await.insert(key.to_string(), n);
                n
            }
        }
    }
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
    pub async fn get_cache_entry(&self, cache_key: &str) -> anyhow::Result<Option<String>> {
        let cache_value = sqlx::query_scalar!(
            "select value from extracted.fetcher_cache where key = ?",
            cache_key
        )
        .fetch_optional(&self.db)
        .await?;
        Ok(cache_value)
    }

    pub async fn set_cache_entry(&self, cache_key: &str, cache_value: &str) -> anyhow::Result<()> {
        let now = Timestamptz(Utc::now());
        sqlx::query!(
            "insert into extracted.fetcher_cache (key, timestamp_unix_ms, value) values (?, ?, ?)",
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
    ) -> anyhow::Result<Vec<SingleExtractedEvent>> {
        self.ensure_time_range_extracted_valid(from, to)
            .await
            .context("updating extracted results")?;

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
    ) -> anyhow::Result<()> {
        let days = self.get_affected_utc_days(from, to);
        {
            let days_str = serde_json::to_string(&days)?;
            let doesnt_need_update: Vec<DateUtc> = sqlx::query_scalar!( r#"select utc_date as "date_utc: _" from extracted.extracted_current where utc_date in (select value from json_each(?)) and extracted_timestamp_unix_ms > raw_events_changed_timestamp_unix_ms"#, days_str).fetch_all(&self.db).await.context("fetching currents")?;
            let doesnt_need_update = HashSet::<DateUtc>::from_iter(doesnt_need_update.into_iter());
            let needs_update: Vec<_> = days
                .into_iter()
                .filter(|e| !doesnt_need_update.contains(e))
                .collect();
            log::debug!("found {} dates that need update", needs_update.len());
            for day in needs_update {
                let now = Timestamptz(Utc::now());
                self.extract_time_range(
                    Timestamptz(day.0.and_hms(0, 0, 0)),
                    Timestamptz((day.0 + chrono::Duration::days(1)).and_hms(0, 0, 0)),
                )
                .await
                .with_context(|| format!("extracting tags for day {:?}", day))?;
                let updated = sqlx::query!("update extracted_current set extracted_timestamp_unix_ms = ? where utc_date = ?", now, day).execute(&self.db).await.with_context(|| format!("updating extracted timestamp {:?} {:?}", day, now))?.rows_affected();
                if updated == 0 {
                    let zero = Timestamptz(Utc.ymd(1970, 1, 1).and_hms(0, 1, 1));
                    sqlx::query!("insert into extracted_current (utc_date, extracted_timestamp_unix_ms, raw_events_changed_timestamp_unix_ms) values (?, ?, ?)", day, now, zero)
                        .execute(&self.db).await
                        .with_context(|| {
                            format!("inserting extracted timestamp {:?} {:?}", day, now)
                        })?;
                }
            }
            Ok(())
        }
    }
    fn get_affected_utc_days(&self, from: &Timestamptz, to: &Timestamptz) -> Vec<DateUtc> {
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

    async fn map_thong(&self, a: DbEvent, r: Tags) -> anyhow::Result<Vec<InExtractedTag>> {
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
        //total_cache_get_dur += now.elapsed();
        let now = Instant::now();
        let (tags, iterations) = get_tags(&self, r).await;
        //total_extract_dur += now.elapsed();
        //total_extract_iterations += iterations;

        Ok(futures::stream::iter(tags.into_iter())
            .flat_map(move |(tag, values)| {
                let timestamp = timestamp.clone();
                async move {
                    let tag = self.tags_cache.get(&tag).await;

                    futures::stream::iter(values).then(move |value| {
                        let timestamp = timestamp.clone();
                        async move {
                            let now = Instant::now();
                            let value = self.values_cache.get(&value).await;
                            //total_cache_get_dur += now.elapsed();
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
    ) -> anyhow::Result<()> {
        log::debug!("extract_time_range {:?} to {:?}", from, to);
        {
            let res = sqlx::query!(
                "delete from extracted_events where timestamp_unix_ms >= ? and timestamp_unix_ms < ?",
                from, to)
            .execute(&self.db).await
            .context("removing stale events")?;
            log::info!("removed {} stale events", res.rows_affected());
        }

        /*let raws = YieldEventsFromTrbttDatabase {
            db: &*self.db_events,
            chunk_size: 1000,
            last_fetched: from.clone(),
            ascending: true,
        };*/
        let raws = sqlx::query_as!(
            DbEvent,
            r#"select
                insertion_sequence, id, timestamp_unix_ms as "timestamp_unix_ms: _",
                data_type, duration_ms, data
            from raw_events.events where timestamp_unix_ms > ? order by timestamp_unix_ms asc"#,
            from
        )
        .fetch(&self.db);

        let now = Instant::now();
        let mut total_raw: usize = 0;
        let mut total_extracted: usize = 0;
        let mut total_tags: usize = 0;
        let mut total_tag_values: usize = 0;
        let mut total_extract_iterations: usize = 0;
        let mut total_extract_dur = Duration::from_secs(0);
        let mut total_cache_get_dur = Duration::from_secs(0);

        let mut extracted: BoxStream<Vec<Result<_, _>>> = Box::pin(
            raws.map_err(|e| anyhow::Error::new(e))
                .try_take_while(|a| futures::future::ready(Ok(&a.timestamp_unix_ms < &to)))
                .try_filter_map(|a| async {
                    //total_raw += 1;
                    let r = a.deserialize_data();
                    let ex: Tags = match r {
                        Ok(r) => {
                            let ex = r.extract_info();
                            match ex {
                                Some(ex) => ex,
                                None => {
                                    return Ok(None);
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("{:#?}", e);
                            return Ok(None);
                        }
                    };

                    Ok(Some((a, ex)))
                })
                .and_then(move |(a, r)| self.map_thong(a, r))
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
            // TODO: transaction
            for ele in chunk.into_iter().flatten().flatten() {
                sqlx::query!("insert into extracted.extracted_events (timestamp_unix_ms, duration_ms, tag, value) values (?, ?, ?, ?)", ele.timestamp_unix_ms, ele.duration_ms, ele.tag, ele.value)
                    .execute(&self.db)
                    .await?;
                updated += 1;
            }
            log::info!("inserted {} ({:?})", updated, now.elapsed());
        }
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
                total_extract_dur.div_f32(total_raw as f32),
                total_cache_get_dur.div_f32(total_raw as f32)
            );
            // log::debug!("cache stats")
        }

        Ok(())
    }
}
