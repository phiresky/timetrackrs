use diesel::prelude::*;
use structopt::StructOpt;
use track_pc_usage_rs::import;

#[derive(StructOpt)]
#[structopt(about = "Import events from a different program")]
enum ImportArgs {
    AppUsage(import::app_usage_sqlite::AppUsageImport),
}

fn main() -> anyhow::Result<()> {
    let opt = ImportArgs::from_args();
    match opt {
        ImportArgs::AppUsage(o) => {
            let data = import::app_usage_sqlite::app_usage_import(o)?;

            println!("inserting...");
            use track_pc_usage_rs::schema::activity;
            let db = track_pc_usage_rs::database::connect()?;

            let updated = diesel::insert_into(activity::table)
                .values(&data)
                .execute(&db)?;

            println!("successfully inserted {}/{} entries", updated, data.len());
        }
    };
    Ok(())
}
