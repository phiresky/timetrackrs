use crate::prelude::*;
use enum_dispatch::enum_dispatch;

use self::tags::Tags;

pub mod fetchers;
pub mod tags;

#[enum_dispatch(EventData)]
pub trait ExtractInfo {
    /// if returns None, event is discarded as (currently) uninteresting
    fn extract_info(&self) -> Option<Tags>;
}
