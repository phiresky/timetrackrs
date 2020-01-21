use track_pc_usage_rs as trbtt;

use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use trbtt::capture::serialize_captured;
use trbtt::sampler::Sampler;
use trbtt::util;

fn main() -> anyhow::Result<()> {
    let db = trbtt::database::connect()?;

    let mut c = trbtt::capture::x11::X11Capturer::init()?;

    use trbtt::models::*;
    use trbtt::schema::activity;

    // println!("{}", serde_json::to_string_pretty(&data)?);
    let sampler = Sampler::RandomSampler { avg_time: 30.0 };
    let sampler_sequence_id = util::random_uuid();
    {
        loop {
            let sample = sampler.get_sample();
            println!("sleeping {}s", sample);
            std::thread::sleep(std::time::Duration::from_secs_f64(sample));

            let res = c.capture()?;

            let (data_type, data_type_version, data) = serialize_captured(&res)?;

            diesel::insert_into(activity::table)
                .values(&NewActivity {
                    id: util::random_uuid(),
                    timestamp: Timestamptz::now(),
                    sampler: sampler.clone(),
                    sampler_sequence_id: sampler_sequence_id.clone(),
                    data_type,
                    data_type_version,
                    data,
                })
                .execute(&db)?;
        }
    }
}
