use diesel::prelude::*;
use import::Importable;
use structopt::StructOpt;
use track_pc_usage_rs as trbtt;
use track_pc_usage_rs::models::NewActivity;
use trbtt::import;

#[derive(StructOpt)]
#[structopt(about = "Import events from a different program")]
enum ImportArgs {
    AppUsage(import::app_usage_sqlite::AppUsageImportArgs),
    Journald(import::journald::JournaldImportArgs),
}

// stupid. how to make shorter?
impl Importable for ImportArgs {
    fn import(&self) -> anyhow::Result<Vec<NewActivity>> {
        use ImportArgs::*;
        match &self {
            AppUsage(a) => a.import(),
            Journald(a) => a.import(),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let opt = ImportArgs::from_args();
    let data = opt.import()?;
    println!("inserting...");
    use track_pc_usage_rs::schema::activity;
    let db = track_pc_usage_rs::database::connect()?;

    let updated = diesel::insert_into(activity::table)
        .values(&data)
        .execute(&db)?;

    println!("successfully inserted {}/{} entries", updated, data.len());
    Ok(())
}
