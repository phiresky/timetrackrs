pub mod x11;
use crate::import::app_usage_sqlite::AppUsageEntry;
use serde::{Deserialize, Serialize};
use typescript_definitions::TypeScriptify;

#[derive(Serialize, TypeScriptify)]
#[serde(tag = "data_type", content = "data")]
pub enum CapturedData {
    x11(x11::X11CapturedData),
    app_usage(AppUsageEntry),
}

// TODO: replace these with just using serde (not easy cause of version)
pub fn serialize_captured(data: &CapturedData) -> anyhow::Result<(String, i32, String)> {
    match data {
        CapturedData::x11(d) => Ok(("x11".to_string(), 2, serde_json::to_string(d)?)),
        CapturedData::app_usage(d) => Ok(("app_usage".to_string(), 1, serde_json::to_string(d)?)),
    }
}

pub fn deserialize_captured(data: (&str, i32, &str)) -> anyhow::Result<CapturedData> {
    Ok(match (data.0, data.1) {
        ("x11", 2) => CapturedData::x11(serde_json::from_str::<x11::X11CapturedData>(&data.2)?),
        ("app_usage", 1) => {
            CapturedData::app_usage(serde_json::from_str::<AppUsageEntry>(&data.2)?)
        }
        _ => Err(anyhow::anyhow!("unknown data type {}@{}", data.0, data.1))?,
    })
}
