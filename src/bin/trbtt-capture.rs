#![warn(clippy::print_stdout)]

use track_pc_usage_rs as trbtt;

use diesel::prelude::*;
use trbtt::prelude::*;

fn main() -> anyhow::Result<()> {
    util::init_logging();

    let args = CaptureArgs::from_args();

    let mut c = args.create_capturer()?;
    let db = trbtt::db::raw_events::connect()?;

    use trbtt::db::models::*;
    use trbtt::db::schema::raw_events::events;

    // println!("{}", serde_json::to_string_pretty(&data)?);
    let duration_ms: i64 = 30000;

    let idgen = libxid::new_generator();

    loop {
        log::info!("sleeping {}s", duration_ms / 1000);
        std::thread::sleep(std::time::Duration::from_millis(duration_ms as u64));

        let data = c.capture()?;
        let act = CreateNewDbEvent {
            id: idgen.new_id().unwrap().encode(),
            timestamp: Utc::now(),
            duration_ms,
            data,
        };
        let ins: NewDbEvent = act.try_into()?;

        diesel::insert_into(events::table)
            .values(&ins)
            .execute(&db)?;
    }
}
