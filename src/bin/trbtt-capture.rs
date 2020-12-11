#![warn(clippy::print_stdout)]

use track_pc_usage_rs as trbtt;

use diesel::prelude::*;
use trbtt::prelude::*;

fn main() -> anyhow::Result<()> {
    util::init_logging();

    let args = CaptureArgs::from_args();

    let mut c = args.create_capturer()?;
    let db = trbtt::db::connect()?;

    use trbtt::db::models::*;
    use trbtt::db::schema::events;

    // println!("{}", serde_json::to_string_pretty(&data)?);
    let sampler = Sampler::Explicit { duration: 30.0 }; //Sampler::RandomSampler { avg_time: 30.0 };
    let sampler_sequence_id = util::random_uuid();

    loop {
        let sample = sampler.get_sample();
        log::info!("sleeping {}s", sample);
        std::thread::sleep(std::time::Duration::from_secs_f64(sample));

        let data = c.capture()?;
        let act = CreateNewDbEvent {
            id: util::random_uuid(),
            timestamp: Utc::now(),
            sampler: sampler.clone(),
            sampler_sequence_id: sampler_sequence_id.clone(),
            data,
        };
        let ins: NewDbEvent = act.try_into()?;

        diesel::insert_into(events::table)
            .values(&ins)
            .execute(&db)?;
    }
}
