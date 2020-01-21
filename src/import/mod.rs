pub mod app_usage_sqlite;
pub mod journald;

use crate::models::NewActivity;

pub trait Importable {
    fn import(&self) -> anyhow::Result<Vec<NewActivity>>;
}
