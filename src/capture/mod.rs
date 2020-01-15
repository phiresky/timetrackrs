pub mod x11;
use serde_json::{Value as J};

pub struct CapturedData {
    pub data_type: String,
    pub data_type_version: i32,
    pub data: J,
}
