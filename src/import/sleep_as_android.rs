use crate::util::iso_string_to_date;

use crate::prelude::*;

#[derive(StructOpt)]
pub struct SleepAsAndroidImportArgs {
    filename: String,
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify)]
pub struct SleepAsAndroidEntry {
    header_row: Vec<String>, // headers (static headers, then times->movement amount, then events)
    data_row: Vec<String>,
    noise_row: Option<Vec<String>>,
}

fn parse_saa_entry(
    header_row: &mut Option<Vec<String>>,
    data_row: &mut Option<Vec<String>>,
    noise_row: &mut Option<Vec<String>>,
) -> anyhow::Result<Option<NewDbEvent>> {
    if let Some(header) = header_row.take() {
        let data = data_row.take().unwrap();
        let noise = noise_row.take();
        let id = format!("sleep_as_android_{}", data[0]); // in theory not globally unique, but very unlikely multiple events started in same millisecond

        let (from, to) = {
            let tz = &data[1];
            let from = &data[2];
            let to = &data[3];
            log::debug!("tz={}, from={}, to={}", tz, from, to);
            let tzz: chrono_tz::Tz = tz.parse().unwrap();

            let from = NaiveDateTime::parse_from_str(from, "%d. %m. %Y %H:%M")?;
            let to = NaiveDateTime::parse_from_str(to, "%d. %m. %Y %H:%M")?;
            let from = tzz
                .from_local_datetime(&from)
                .earliest() // if summer time switch is at 3am, it's more likely the person went to bed earlier
                .expect("impossible time");
            let to = tzz.from_local_datetime(&to).single().unwrap();
            (from, to)
        };

        Ok(Some(
            CreateNewDbEvent {
                id,
                data: EventData::sleep_as_android_v1(SleepAsAndroidEntry {
                    header_row: header,
                    data_row: data,
                    noise_row: noise,
                }),
                timestamp: from.with_timezone(&chrono::Utc),
                sampler: Sampler::Explicit {
                    duration: (to.signed_duration_since(from).num_milliseconds() as f64) / 1000.0,
                },
                sampler_sequence_id: "".to_string(),
            }
            .try_into()?,
        ))
    } else {
        Ok(None)
    }
}

impl Importable for SleepAsAndroidImportArgs {
    fn import(&self) -> anyhow::Result<Vec<NewDbEvent>> {
        let mut entries = Vec::new();
        let mut csv = csv::ReaderBuilder::new()
            .has_headers(false)
            .flexible(true) // cols depends on number of events
            .from_reader(File::open(&self.filename)?);

        let mut offset_from_id_row = -9999;
        let mut header_row: Option<Vec<String>> = None;
        let mut data_row: Option<Vec<String>> = None;
        let mut noise_row: Option<Vec<String>> = None;
        for result in csv.records() {
            let record = result?;
            if record.get(0) == Some("Id") {
                offset_from_id_row = 0;
                parse_saa_entry(&mut header_row, &mut data_row, &mut noise_row)?
                    .map(|e| entries.push(e));
            }
            match offset_from_id_row {
                0 => header_row = Some(record.iter().map(|e| e.to_string()).collect()),
                1 => data_row = Some(record.iter().map(|e| e.to_string()).collect()),
                2 => noise_row = Some(record.iter().map(|e| e.to_string()).collect()),
                _ => panic!("more than 3 rows??"),
            }
            offset_from_id_row += 1;
        }

        // last row
        parse_saa_entry(&mut header_row, &mut data_row, &mut noise_row)?.map(|e| entries.push(e));

        Ok(entries)
    }
}

impl ExtractInfo for SleepAsAndroidEntry {
    fn extract_info(&self) -> Option<ExtractedInfo> {
        Some(ExtractedInfo::PhysicalActivity {
            activity_type: "sleeping".to_string(),
        })
    }
}
