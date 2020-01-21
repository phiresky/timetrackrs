pub mod x11;
use crate::import::app_usage_sqlite::AppUsageEntry;
use crate::import::journald::JournaldEntry;
use crate::prelude::*;
use enum_dispatch::enum_dispatch;
use serde::Serialize;
use typescript_definitions::TypeScriptify;
use x11::X11CapturedData;

#[enum_dispatch]
#[derive(Serialize, TypeScriptify)]
#[serde(tag = "data_type", content = "data")]
#[allow(non_camel_case_types)]
pub enum CapturedData {
    x11_v2(X11CapturedData),
    app_usage_v1(AppUsageEntry),
    journald(JournaldEntry),
}

// todo: maybe borrow more here
pub struct CreateNewActivity {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub sampler: Sampler,
    pub sampler_sequence_id: String,
    pub data: CapturedData,
}

impl std::convert::TryFrom<CreateNewActivity> for NewActivity {
    type Error = anyhow::Error;

    fn try_from(value: CreateNewActivity) -> Result<Self, Self::Error> {
        let (data_type, data) = match &value.data {
            CapturedData::x11_v2(d) => ("x11_v2", serde_json::to_string(d)?),
            CapturedData::app_usage_v1(d) => ("app_usage_v1", serde_json::to_string(d)?),
            CapturedData::journald(d) => ("journald_v1", serde_json::to_string(d)?),
        };
        Ok(NewActivity {
            id: value.id,
            timestamp: Timestamptz::new(value.timestamp),
            sampler: value.sampler,
            sampler_sequence_id: value.sampler_sequence_id,
            data_type: data_type.to_string(),
            data,
        })
    }
}

pub fn deserialize_captured((data_type, data): (&str, &str)) -> anyhow::Result<CapturedData> {
    Ok(match data_type {
        "x11_v2" => CapturedData::x11_v2(serde_json::from_str::<x11::X11CapturedData>(data)?),
        "app_usage_v1" => CapturedData::app_usage_v1(serde_json::from_str::<AppUsageEntry>(data)?),
        _ => anyhow::bail!("unknown data type {}", data_type),
    })
}
