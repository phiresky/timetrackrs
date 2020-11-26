use crate::util::iso_string_to_date;

use crate::prelude::*;

#[derive(StructOpt)]
pub struct SleepAsAndroidImportArgs {
    filename: String,
}
impl Importable for SleepAsAndroidImportArgs {
    fn import(&self) -> anyhow::Result<Vec<NewDbEvent>> {
        let mut csv = csv::ReaderBuilder::new()
            .has_headers(false)
            .flexible(true) // cols depends on number of events
            .from_reader(File::open(&self.filename)?);

        let mut next_is_interesting = false;
        for result in csv.records() {
            let record = result?;
            if record.get(0) == Some("Id") {
                println!("has id");
                next_is_interesting = true;
            } else {
                if next_is_interesting {
                    let tz = record.get(1).unwrap();
                    let from = record.get(2).unwrap();
                    let to = record.get(3).unwrap();
                    println!("tz={}, from={}, to={}", tz, from, to);
                    let tzz: chrono_tz::Tz = tz.parse().unwrap();

                    let from = NaiveDateTime::parse_from_str(from, "%d. %m. %Y %H:%M")?;
                    let to = NaiveDateTime::parse_from_str(to, "%d. %m. %Y %H:%M")?;
                    let from = tzz
                        .from_local_datetime(&from)
                        .single()
                        .expect("fuck you i'm sure this never happens");
                    let to = tzz.from_local_datetime(&to).single().unwrap();
                    println!("{}: {} {}", tz, from, to);
                }
                next_is_interesting = false;
            }
        }

        Ok(vec![])
    }
}
