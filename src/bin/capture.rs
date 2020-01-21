use track_pc_usage_rs as trbtt;

use diesel::prelude::*;
use trbtt::prelude::*;

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

            let data = c.capture()?;
            let act = CreateNewActivity {
                id: util::random_uuid(),
                timestamp: Utc::now(),
                sampler: sampler.clone(),
                sampler_sequence_id: sampler_sequence_id.clone(),
                data,
            };
            let ins: NewActivity = act.try_into()?;

            diesel::insert_into(activity::table)
                .values(&ins)
                .execute(&db)?;
        }
    }
}
