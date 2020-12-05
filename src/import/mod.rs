pub mod app_usage_sqlite;
// pub mod google_fitness;
pub mod journald;
pub mod sleep_as_android;
pub mod trbtt;

use crate::prelude::*;

use enum_dispatch::enum_dispatch;

use structopt::StructOpt;

#[enum_dispatch]
#[derive(StructOpt)]
#[structopt(about = "Import events from a different program")]
pub enum ImportArgs {
    AppUsage(app_usage_sqlite::AppUsageImportArgs),
    Journald(journald::JournaldImportArgs),
    Trbtt(trbtt::TrbttImportArgs),
    SleepAsAndroid(sleep_as_android::SleepAsAndroidImportArgs),
}

#[enum_dispatch(ImportArgs)]
pub trait Importable {
    fn import(&self) -> ImportResult;
}


pub type ImportResult = anyhow::Result<Box<dyn Iterator<Item=Vec<NewDbEvent>>>>;