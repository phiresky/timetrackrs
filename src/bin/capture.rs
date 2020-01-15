use track_pc_usage_rs as trbtt;

use diesel::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use trbtt::capture::serialize_captured;

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
    let db = trbtt::database::connect()?;

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

            let (data_type, data_type_version, data) = serialize_captured(&res)?;

            diesel::insert_into(activity::table)
                .values(&NewActivity {
                    timestamp: Timestamptz::now(),
                    sampler: serde_json::to_string(&sampler)?,
                    data_type,
                    data_type_version,
                    data,
                })
                .execute(&db)?;
        }
    }
}
