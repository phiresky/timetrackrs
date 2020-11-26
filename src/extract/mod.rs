use crate::prelude::*;
use enum_dispatch::enum_dispatch;

pub mod properties;
pub mod tags;

#[enum_dispatch(EventData)]
pub trait ExtractInfo {
    /// if returns None, event is discarded as (currently) uninteresting
    fn extract_info(&self) -> Option<ExtractedInfo>;
}
