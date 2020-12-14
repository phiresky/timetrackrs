use std::{collections::HashSet, time::Instant};

use crate::{api::SingleExtractedEvent, prelude::*};
use diesel::SqliteConnection;
use diesel::{prelude::*, sql_types::Text};
use itertools::Itertools;
use rocket::request::FromRequest;
use rocket_contrib::database;

use std::iter::FromIterator;
#[database("raw_events_database")]
pub struct DbEvents(diesel::SqliteConnection);

#[database("config_database")]
pub struct DbConfig(diesel::SqliteConnection);

#[database("extracted_database")]
pub struct DbExtracted(diesel::SqliteConnection);

pub struct DatyBasy {
    pub db_events: DbEvents,
    pub db_config: DbConfig,
    pub db_extracted: DbExtracted,
    enabled_tag_rules: Vec<TagRule>,
}

impl<'a, 'r> FromRequest<'a, 'r> for DatyBasy {
    type Error = ();

    fn from_request(request: &'a rocket::Request<'r>) -> rocket::request::Outcome<Self, ()> {
        let db_extracted = DbExtracted::from_request(request)?;
        if let Err(e) = crate::db::extracted::migrate(&db_extracted) {
            log::error!("{:#?}", e);
            return rocket::request::Outcome::Failure((
                rocket::http::Status::InternalServerError,
                (),
            ));
        };
        let db_events = DbEvents::from_request(request)?;
        if let Err(e) = crate::db::raw_events::migrate(&db_events) {
            log::error!("{:#?}", e);
            return rocket::request::Outcome::Failure((
                rocket::http::Status::InternalServerError,
                (),
            ));
        };
        let db_config = DbConfig::from_request(request)?;
        if let Err(e) = crate::db::config::migrate(&db_config) {
            log::error!("{:#?}", e);
            return rocket::request::Outcome::Failure((
                rocket::http::Status::InternalServerError,
                (),
            ));
        };
        rocket::request::Outcome::Success(DatyBasy {
            enabled_tag_rules: match fetch_tag_rules(&db_config) {
                Ok(r) => r,
                Err(e) => {
                    log::error!("{:#?}", e);
                    return rocket::request::Outcome::Failure((
                        rocket::http::Status::InternalServerError,
                        (),
                    ));
                }
            },
            db_events,
            db_extracted,
            db_config,
        })
    }
}

fn fetch_tag_rules(db_config: &SqliteConnection) -> anyhow::Result<Vec<TagRule>> {
    use crate::db::schema::config::tag_rule_groups::dsl::*;
    let groups: Vec<TagRuleGroup> = tag_rule_groups.load(db_config)?;
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
        .flat_map(|g| g.data.into_iter_active_rules())
        .collect())
}
impl DatyBasy {
    /*pub fn new(conn: &'a SqliteConnection) -> DatyBasy {
        DatyBasy {
            conn,
            enabled_tag_rules: None,
        }
    }*/

    pub fn get_cache_entry(&self, cache_key: &str) -> anyhow::Result<Option<String>> {
        use crate::db::schema::extracted::fetcher_cache::dsl::*;

        let cache_value = fetcher_cache
            .find(cache_key)
            .select(value)
            .first::<String>(&self.db_extracted.0)
            .optional()?;
        Ok(cache_value)
    }

    pub fn set_cache_entry(&self, cache_key: &str, cache_value: &str) -> anyhow::Result<()> {
        use crate::db::schema::extracted::fetcher_cache::dsl::*;
        diesel::insert_into(fetcher_cache)
            .values((
                key.eq(cache_key),
                timestamp.eq(Timestamptz(Utc::now())),
                value.eq(cache_value),
            ))
            .execute(&*self.db_extracted)
            .context("insert into fetcher_cache db")?;

        Ok(())
    }

    pub fn get_all_tag_rules(&self) -> &[TagRule] {
        &self.enabled_tag_rules
    }

    pub fn get_extracted_for_time_range(
        &self,
        from: &Timestamptz,
        to: &Timestamptz,
    ) -> anyhow::Result<Vec<SingleExtractedEvent>> {
        self.ensure_time_range_extracted_valid(from, to)
            .context("updating extracted results")?;
        use crate::db::schema::extracted::extracted_events::dsl::*;

        let now = Instant::now();
        let q = extracted_events
            .filter(timestamp.ge(from))
            .filter(timestamp.lt(to))
            .order(rowid.asc())
            .load::<OutExtractedTag>(&*self.db_extracted)
            .context("querying extracted db")?;
        let ee = q.into_iter().group_by(|e| e.event_id.clone());
        let e: Vec<_> = ee
            .into_iter()
            .map(|(id, group)| {
                let mut group = group.peekable();
                SingleExtractedEvent {
                    id: id.clone(),
                    timestamp: group.peek().unwrap().timestamp.clone(),
                    duration: group.peek().unwrap().duration,
                    tags: group.map(|e| (e.tag, e.value)).collect(),
                }
            })
            .collect();
        log::debug!("geting extracted from db took {:?}", now.elapsed());
        Ok(e)
    }
    pub fn ensure_time_range_extracted_valid(
        &self,
        from: &Timestamptz,
        to: &Timestamptz,
    ) -> anyhow::Result<()> {
        let days = self.get_affected_utc_days(from, to);
        {
            use crate::db::schema::extracted::extracted_current::dsl::*;
            let doesnt_need_update = extracted_current
                .filter(utc_date.eq_any(&days))
                .filter(extracted_timestamp.gt(raw_events_changed_timestamp))
                .select(utc_date)
                .load::<DateUtc>(&*self.db_extracted)
                .context("fetching currents")?;
            let doesnt_need_update = HashSet::<DateUtc>::from_iter(doesnt_need_update.into_iter());
            let needs_update: Vec<_> = days
                .into_iter()
                .filter(|e| !doesnt_need_update.contains(e))
                .collect();
            log::debug!("found {} dates that need update", needs_update.len());
            for day in needs_update {
                let now = Timestamptz(Utc::now());
                self.extract_time_range(
                    &Timestamptz(day.0.and_hms(0, 0, 0)),
                    &Timestamptz((day.0 + chrono::Duration::days(1)).and_hms(0, 0, 0)),
                )
                .with_context(|| format!("extracting tags for day {:?}", day))?;
                let updated = diesel::update(extracted_current.filter(utc_date.eq(&day)))
                    .set(extracted_timestamp.eq(&now))
                    .execute(&*self.db_extracted)
                    .with_context(|| format!("updating extracted timestamp {:?} {:?}", day, now))?;
                if updated == 0 {
                    let zero = Timestamptz(Utc.ymd(1970, 1, 1).and_hms(0, 1, 1));
                    diesel::insert_into(extracted_current)
                        .values(vec![(
                            utc_date.eq(&day),
                            extracted_timestamp.eq(&now),
                            raw_events_changed_timestamp.eq(zero),
                        )])
                        .execute(&*self.db_extracted)
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
        return affected;
    }

    pub fn extract_time_range(&self, from: &Timestamptz, to: &Timestamptz) -> anyhow::Result<()> {
        log::debug!("extract_time_range {:?} to {:?}", from, to);
        {
            use crate::db::schema::extracted::extracted_events::dsl::*;
            let res = diesel::sql_query(
                "delete from extracted_events where timestamp >= ? and timestamp < ?",
            )
            .bind::<Text, _>(&from)
            .bind::<Text, _>(&to)
            .execute(&*self.db_extracted)
            .context("removing stale events")?;
            log::info!("removed {} stale events", res);
        }

        let raws = YieldEventsFromTrbttDatabase {
            db: &*self.db_events,
            chunk_size: 1000,
            last_fetched: from.clone(),
            ascending: true,
        };
        let now = Instant::now();
        let mut total_raw = 0;
        let mut total_extracted = 0;
        let mut total_tags = 0;
        let mut total_tag_values = 0;
        let extracted = raws
            .flatten()
            .take_while(|a| &a.timestamp < to)
            .filter_map(|a| {
                total_raw += 1;
                let r = a
                    .deserialize_data()
                    .map_err(|e| log::warn!("{:#?}", e))
                    .ok()?;
                Some((a, r.extract_info()?))
            })
            .flat_map(|(a, r)| {
                total_extracted += 1;
                total_tags += r.tag_count();
                total_tag_values += r.total_value_count();
                let timestamp = a.timestamp.clone();
                let id = a.id.clone();
                let duration = a.sampler.get_duration();
                let tags = get_tags(&self, r);

                tags.into_iter().flat_map(move |(tag, values)| {
                    let tag = tag.clone();
                    let timestamp = timestamp.clone();
                    let id = id.clone();
                    values.into_iter().map(move |value| InExtractedTag {
                        timestamp: (&timestamp).clone(),
                        duration,
                        event_id: (&id).clone(),
                        tag: tag.clone(),
                        value,
                    })
                })
            })
            .chunks(10000);

        for chunk in extracted.into_iter() {
            use crate::db::schema::extracted::extracted_events::dsl::*;
            let chunk: Vec<_> = chunk.collect();
            let updated = diesel::insert_into(extracted_events)
                .values(&chunk)
                .execute(&*self.db_extracted)
                .context("inserting new extracted events into db")?;
            log::info!("inserted {}", updated);
        }
        log::debug!(
            "extraction yielded {} of {} seen events with {} tags with {} values total. extracting tags took {:?}",
            total_extracted,
            total_raw,
            total_tags,
            total_tag_values,
            now.elapsed()
        );

        Ok(())
    }
}
