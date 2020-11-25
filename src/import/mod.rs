pub mod app_usage_sqlite;
pub mod google_fitness;
pub mod journald;

use crate::prelude::*;

use app_usage_sqlite::AppUsageImportArgs;
use enum_dispatch::enum_dispatch;
use journald::JournaldImportArgs;
use structopt::StructOpt;

#[enum_dispatch]
#[derive(StructOpt)]
#[structopt(about = "Import events from a different program")]
pub enum ImportArgs {
    AppUsage(AppUsageImportArgs),
    Journald(JournaldImportArgs),
}

#[enum_dispatch(ImportArgs)]
pub trait Importable {
    fn import(&self) -> anyhow::Result<Vec<NewDbEvent>>;
}
