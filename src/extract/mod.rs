use crate::capture::CapturedData;
use crate::extract::properties::ExtractedInfo;

pub mod properties;

pub trait ExtractInfo {
    /// if returns None, event is discarded as (currently) uninteresting
    fn extract_info(&self, event_id: String) -> Option<ExtractedInfo>;
}

impl ExtractInfo for CapturedData {
    fn extract_info(&self, event_id: String) -> Option<ExtractedInfo> {
        match self {
            CapturedData::x11(x) => x.extract_info(event_id),
            CapturedData::app_usage(x) => x.extract_info(event_id),
        }
    }
}
