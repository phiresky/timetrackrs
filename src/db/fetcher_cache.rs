use crate::prelude::*;
use diesel::prelude::*;
use diesel::SqliteConnection;

pub fn get_cache_entry(
    db: &mut SqliteConnection,
    cache_key: &str,
) -> anyhow::Result<Option<String>> {
    use crate::db::schema::fetcher_cache::dsl::*;

    let cache_value = fetcher_cache
        .find(cache_key)
        .select(value)
        .first::<String>(db)
        .optional()?;
    Ok(cache_value)
}

pub fn set_cache_entry(
    db: &mut SqliteConnection,
    cache_key: &str,
    cache_value: &str,
) -> anyhow::Result<()> {
    use crate::db::schema::fetcher_cache::dsl::*;
    diesel::insert_into(fetcher_cache)
        .values((
            key.eq(cache_key),
            timestamp.eq(Timestamptz::new(Utc::now())),
            value.eq(cache_value),
        ))
        .execute(db)
        .context("insert into fetcher_cache db")?;

    Ok(())
}
