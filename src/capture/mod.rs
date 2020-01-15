pub mod x11;
use serde::{Deserialize, Serialize};
use typescript_definitions::TypeScriptify;

pub struct Captured {
    pub data_type: String, // TODO: this should be the key in CapturedData
    pub data_type_version: i32,
    pub data: CapturedData,
}

#[derive(Serialize, TypeScriptify)]
#[serde(tag = "data_type", content = "data")]
pub enum CapturedData {
    x11(x11::X11CapturedData),
}

pub fn serialize_captured(data: &CapturedData) -> anyhow::Result<(String, i32, String)> {
    match data {
        CapturedData::x11(d) => Ok(("x11".to_string(), 2, serde_json::to_string(d)?)),
    }
}
