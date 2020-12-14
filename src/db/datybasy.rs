use crate::prelude::*;
use diesel::prelude::*;
use diesel::SqliteConnection;
use rocket::request::FromRequest;
use rocket_contrib::database;

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
        use crate::db::schema::config::fetcher_cache::dsl::*;

        let cache_value = fetcher_cache
            .find(cache_key)
            .select(value)
            .first::<String>(&self.db_extracted.0)
            .optional()?;
        Ok(cache_value)
    }

    pub fn set_cache_entry(&self, cache_key: &str, cache_value: &str) -> anyhow::Result<()> {
        use crate::db::schema::config::fetcher_cache::dsl::*;
        diesel::insert_into(fetcher_cache)
            .values((
                key.eq(cache_key),
                timestamp.eq(Timestamptz::new(Utc::now())),
                value.eq(cache_value),
            ))
            .execute(&*self.db_extracted)
            .context("insert into fetcher_cache db")?;

        Ok(())
    }

    pub fn get_all_tag_rules(&self) -> &[TagRule] {
        &self.enabled_tag_rules
    }
}
