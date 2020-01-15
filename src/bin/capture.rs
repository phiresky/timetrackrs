use track_pc_usage_rs as trbtt;

use diesel::prelude::*;
use dotenv::dotenv;
use rand::Rng;
use serde::{Serialize, Deserialize};
use std::env;

#[macro_use]
extern crate diesel_migrations;
use diesel_migrations::embed_migrations;
embed_migrations!();

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Sampler {
    RandomSampler { avg_time: f64 },
}

fn get_sample(s: &Sampler) -> f64 {
    match s {
        Sampler::RandomSampler { avg_time } => {
            let distribution = rand::distributions::Uniform::new(0f64, (avg_time) * 2.0);
            let mut rng = rand::thread_rng();
            return rng.sample(distribution);
        }
    }
}

fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = SqliteConnection::establish(&database_url)?;
    db.execute("pragma page_size = 32768;")?;
    db.execute("pragma foreign_keys = on;")?;
    db.execute("pragma temp_store = memory;")?;
    db.execute("pragma journal_mode = WAL;")?;
    db.execute("pragma synchronous = normal;")?;
    db.execute("pragma mmap_size= 30000000000;")?;
    db.execute("pragma auto_vacuum = incremental")?;
    db.execute("pragma optimize;")?;

    embedded_migrations::run_with_output(&db, &mut std::io::stdout())?;

    let mut c = trbtt::capture::x11::X11Capturer::init()?;

    use trbtt::models::*;
    use trbtt::schema::activity;

    // println!("{}", serde_json::to_string_pretty(&data)?);
    let sampler = Sampler::RandomSampler { avg_time: 60.0 };
    {
        loop {
            let sample = get_sample(&sampler);
            println!("sleeping {}s", sample);
            std::thread::sleep(std::time::Duration::from_secs_f64(sample));

            let res = c.capture()?;

            diesel::insert_into(activity::table)
                .values(&NewActivity {
                    timestamp: Timestamptz::now(),
                    sampler: serde_json::to_string(&sampler)?,
                    data_type: res.data_type,
                    data_type_version: res.data_type_version,
                    data: res.data.to_string(),
                })
                .execute(&db)?;

            
        }
    }
}
