use crate::prelude::*;
use diesel::prelude::*;
use diesel::SqliteConnection;

pub struct DatyBasy<'a> {
    conn: &'a mut SqliteConnection,
    enabled_tag_rules: Option<Vec<TagRule>>,
}

impl<'a> DatyBasy<'a> {
    pub fn new(conn: &mut SqliteConnection) -> DatyBasy {
        DatyBasy {
            conn,
            enabled_tag_rules: None,
        }
    }

    pub fn get_cache_entry(&self, cache_key: &str) -> anyhow::Result<Option<String>> {
        use crate::db::schema::fetcher_cache::dsl::*;

        let cache_value = fetcher_cache
            .find(cache_key)
            .select(value)
            .first::<String>(self.conn)
            .optional()?;
        Ok(cache_value)
    }

    pub fn set_cache_entry(&self, cache_key: &str, cache_value: &str) -> anyhow::Result<()> {
        use crate::db::schema::fetcher_cache::dsl::*;
        diesel::insert_into(fetcher_cache)
            .values((
                key.eq(cache_key),
                timestamp.eq(Timestamptz::new(Utc::now())),
                value.eq(cache_value),
            ))
            .execute(self.conn)
            .context("insert into fetcher_cache db")?;

        Ok(())
    }

    pub fn fetch_all_tag_rules_if_thoink(&mut self) -> anyhow::Result<()> {
        use crate::db::schema::tag_rule_groups::dsl::*;
        if self.enabled_tag_rules.is_none() {
            let groups: Vec<TagRuleGroup> = tag_rule_groups.load(self.conn)?;
            /*if groups.len() == 0 {
                // insert defaults
                let groups = 
                diesel::insert_into(tag_rule_groups)
                    .values(groups)
                    .execute(self.conn)?;
                return self.fetch_all_tag_rules_if_thoink();
            }*/
            self.enabled_tag_rules.replace(
                groups
                    .into_iter().chain(get_default_tag_rule_groups().into_iter())
                    .flat_map(|g| g.data.into_iter_active_rules())
                    .collect(),
            );
        }
        Ok(())
    }

    pub fn get_all_tag_rules(&self) -> anyhow::Result<&[TagRule]> {
        Ok(&self
            .enabled_tag_rules
            .as_ref()
            .expect("forgot to call fetch first"))
    }
}
