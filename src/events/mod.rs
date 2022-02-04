use crate::prelude::*;

// todo: auto generate this enum (with dependency injection?)
#[enum_dispatch]
#[derive(Serialize, TypeScriptify, Debug, Deserialize, Clone)]
#[serde(tag = "data_type", content = "data")]
#[allow(non_camel_case_types)]
pub enum EventData {
    x11_v2(X11EventData),
    windows_v1(WindowsEventData),
    macos_v1(MacOSEventData),
    app_usage_v2(AppUsageEntry),
    journald_v1(JournaldEntry),
    sleep_as_android_v1(SleepAsAndroidEntry),
}

// todo: maybe borrow more here
pub struct CreateNewDbEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: i64,
    pub data: EventData,
}

// CreateNewDbEvent.into::<NewDbEvent>
impl std::convert::TryFrom<CreateNewDbEvent> for NewDbEvent {
    type Error = anyhow::Error;

    fn try_from(value: CreateNewDbEvent) -> Result<Self, Self::Error> {
        let (data_type, data) = match &value.data {
            EventData::x11_v2(d) => ("x11_v2", serde_json::to_string(d)?),
            EventData::windows_v1(d) => ("windows_v1", serde_json::to_string(d)?),
            EventData::macos_v1(d) => ("macos_v1", serde_json::to_string(d)?),
            EventData::app_usage_v2(d) => ("app_usage_v2", serde_json::to_string(d)?),
            EventData::journald_v1(d) => ("journald_v1", serde_json::to_string(d)?),
            EventData::sleep_as_android_v1(d) => ("sleep_as_android_v1", serde_json::to_string(d)?),
        };
        Ok(NewDbEvent {
            id: value.id,
            timestamp_unix_ms: Timestamptz(value.timestamp),
            duration_ms: value.duration_ms,
            data_type: data_type.to_string(),
            data,
        })
    }
}

pub fn deserialize_captured((data_type, data): (&str, &str)) -> anyhow::Result<EventData> {
    Ok(match data_type {
        "x11_v2" => serde_json::from_str::<X11EventData>(data)?.into(),
        "windows_v1" => serde_json::from_str::<WindowsEventData>(data)?.into(),
        "macos_v1" => serde_json::from_str::<MacOSEventData>(data)?.into(),
        "app_usage_v2" => serde_json::from_str::<AppUsageEntry>(data)?.into(),
        "journald_v1" => serde_json::from_str::<JournaldEntry>(data)?.into(),
        "sleep_as_android_v1" => serde_json::from_str::<SleepAsAndroidEntry>(data)?.into(),
        _ => anyhow::bail!("unknown data type {}", data_type),
    })
}
