use crate::prelude::*;

// todo: auto generate this enum (with dependency injection?)
#[enum_dispatch]
#[derive(Serialize, TypeScriptify)]
#[serde(tag = "data_type", content = "data")]
#[allow(non_camel_case_types)]
pub enum EventData {
    x11_v2(X11EventData),
    app_usage_v1(AppUsageEntry),
    journald(JournaldEntry),
}

// todo: maybe borrow more here
pub struct CreateNewDbEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub sampler: Sampler,
    pub sampler_sequence_id: String,
    pub data: EventData,
}

// CreateNewDbEvent.into::<NewDbEvent>
impl std::convert::TryFrom<CreateNewDbEvent> for NewDbEvent {
    type Error = anyhow::Error;

    fn try_from(value: CreateNewDbEvent) -> Result<Self, Self::Error> {
        let (data_type, data) = match &value.data {
            EventData::x11_v2(d) => ("x11_v2", serde_json::to_string(d)?),
            EventData::app_usage_v1(d) => ("app_usage_v1", serde_json::to_string(d)?),
            EventData::journald(d) => ("journald_v1", serde_json::to_string(d)?),
        };
        Ok(NewDbEvent {
            id: value.id,
            timestamp: Timestamptz::new(value.timestamp),
            sampler: value.sampler,
            sampler_sequence_id: value.sampler_sequence_id,
            data_type: data_type.to_string(),
            data,
        })
    }
}

pub fn deserialize_captured((data_type, data): (&str, &str)) -> anyhow::Result<EventData> {
    Ok(match data_type {
        "x11_v2" => serde_json::from_str::<X11EventData>(data)?.into(),
        "app_usage_v1" => serde_json::from_str::<AppUsageEntry>(data)?.into(),
        "journald_v1" => serde_json::from_str::<JournaldEntry>(data)?.into(),
        _ => anyhow::bail!("unknown data type {}", data_type),
    })
}
