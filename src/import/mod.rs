pub mod app_usage_sqlite;
// pub mod google_fitness;
pub mod journald;
pub mod sleep_as_android;
pub mod timetrackrs_db;

use crate::prelude::*;

use enum_dispatch::enum_dispatch;

use futures::stream::BoxStream;
use structopt::StructOpt;

#[enum_dispatch]
#[derive(StructOpt)]
#[structopt(about = "Import events from a different program")]
pub enum ImportArgs {
    AppUsage(app_usage_sqlite::AppUsageImportArgs),
    Journald(journald::JournaldImportArgs),
    Timetrackrs(timetrackrs_db::TimetrackrsImportArgs),
    SleepAsAndroid(sleep_as_android::SleepAsAndroidImportArgs),
}

#[async_trait]
#[enum_dispatch(ImportArgs)]
pub trait Importable {
    async fn import(&self) -> ImportResult;
}

pub type ImportResult<'a> = anyhow::Result<BoxStream<'a, anyhow::Result<Vec<NewDbEvent>>>>;
