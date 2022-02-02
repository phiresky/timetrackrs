use std::collections::HashMap;

use crate::prelude::*;

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct SingleExtractedChunk {
    pub from: Timestamptz,
    pub to_exclusive: Timestamptz,
    pub tags: Vec<(String, String, i64)>,
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct SingleExtractedEventWithRaw {
    pub id: String,
    pub timestamp_unix_ms: Timestamptz,
    pub duration_ms: i64,
    pub tags: Tags,
    pub raw: Option<EventData>,
    pub tags_reasons: Option<HashMap<String, TagAddReason>>,
}
#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct TimeRangeRequest {
    pub before: Timestamptz,
    pub after: Timestamptz,
    pub tag: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct InvalidateRangeRequest {
    pub from: Timestamptz,
    pub to: Timestamptz,
}

macro_rules! make_thingois {
    (pub enum $name:ident {
        $($r:ident { request: $req:ty, response: $resp:ty }),+
    }) => (
        #[allow(non_camel_case_types)]
        #[derive(TypeScriptify, Serialize)]
        #[serde(tag="type")]
        pub enum $name {
            $( $r { request: $req, response: $resp }),*
        }
        #[allow(non_snake_case)]
        pub mod Api {
            $(

                pub mod $r {
                    pub use super::super::*;
                    #[allow(non_camel_case_types)]
                    pub type request = $req;
                    #[allow(non_camel_case_types)]
                    pub type response = DebugRes<ApiResponse<$resp>>;
                }
            )*
        }
    )
}

// type Json<T> = warp::reply::Json;
type DebugRes<T> = Result<T, anyhow::Error>;

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct SingleEventsRequest {
    #[serde(deserialize_with = "crate::util::comma_separated")]
    pub ids: Vec<String>,
    pub include_raw: bool,
    pub include_reasons: bool,
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
///
/// search for the timestamp of the next existing raw event, starting at `from` and searching backwards / forwards
///
pub struct TimestampSearchRequest {
    pub backwards: bool,
    pub from: Option<Timestamptz>,
}

make_thingois! {
    pub enum ApiTypesTS {
        time_range {
            request: TimeRangeRequest,
            response: Vec<SingleExtractedChunk>
        },
        single_events {
            request: SingleEventsRequest,
            response: Vec<SingleExtractedEventWithRaw>
        },
        rule_groups {
            request: (),
            response: Vec<TagRuleGroup>
        },
        invalidate_extractions {
            request: InvalidateRangeRequest,
            response: ()
        },
        update_rule_groups {
            request: Vec<TagRuleGroup>,
            response: ()
        },
        get_known_tags {
            request: (),
            response: Vec<String>
        },
        timestamp_search {
            request: TimestampSearchRequest,
            response: Option<Timestamptz>
        }
    }
}

// https://stackoverflow.com/a/40573155/2639190 :(

// type X = ApiTypes::time_range::response;
