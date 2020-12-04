use crate::prelude::*;

pub fn unix_epoch_millis_to_date(timestamp: i64) -> DateTime<Utc> {
    let timestamp_s = timestamp / 1000;
    let timestamp_us = (timestamp % 1000) * 1_000_000;
    let naive_datetime = NaiveDateTime::from_timestamp(timestamp_s, timestamp_us as u32);
    DateTime::from_utc(naive_datetime, Utc)
}

fn timestamp_to_iso_string(timestamp: i64) -> String {
    unix_epoch_millis_to_date(timestamp).to_rfc3339()
}

pub fn iso_string_to_date(s: &str) -> anyhow::Result<DateTime<Utc>> {
    Ok(DateTime::<chrono::FixedOffset>::parse_from_rfc3339(s)?.with_timezone(&chrono::Utc))
}

pub fn random_uuid() -> String {
    uuid::Uuid::new_v4().to_hyphenated().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeScriptify)]
pub struct OsInfo {
    // e.g. "Arch Linux" or "Windows"
    pub os_type: String,
    pub version: String,
    pub batteries: Option<i32>, // useful for determining pc vs laptop
    pub hostname: String,
    pub machine_id: Option<String>,
}
// remove defaults after rewrite
impl Default for OsInfo {
    fn default() -> OsInfo {
        OsInfo {
            os_type: "Arch Linux".to_string(),
            version: "rolling".to_string(),
            batteries: Some(0),
            hostname: "phirearch".to_string(),
            machine_id: None,
        }
    }
}
impl OsInfo {
    pub fn to_partial_general_software(&self, tags: &mut Tags) -> () {
        tags.insert(format!("device-os-type:{}", self.os_type));
        tags.insert(format!("device-os-version:{}", self.version));
        tags.insert(format!("device-hostname:{}", self.hostname));
        self.machine_id
            .as_ref()
            .map(|m| tags.insert(format!("device-machine-id:{}", m)));
        tags.insert(format!(
            "device-type:{}",
            if self.batteries.unwrap_or(0) > 0 {
                SoftwareDeviceType::Laptop
            } else {
                SoftwareDeviceType::Desktop
            }
        ));
    }
}

pub fn get_os_info() -> OsInfo {
    let os_info1 = os_info::get();
    let batteries = battery::Manager::new()
        .and_then(|e| e.batteries())
        .map(|e| e.count() as i32)
        .ok();
    let machine_id = std::fs::read_to_string("/etc/machine-id")
        .map(|e| e.trim().to_string())
        .ok();
    OsInfo {
        os_type: os_info1.os_type().to_string(),
        version: format!("{}", os_info1.version()),
        hostname: hostname::get()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or("".to_string()),
        machine_id,
        batteries: batteries,
    }
}

pub fn init_logging() {
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();
}
