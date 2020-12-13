use std::collections::HashMap;

use crate::prelude::*;

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct SingleExtractedEvent {
    pub id: String,
    pub timestamp: Timestamptz,
    pub duration: f64,
    pub tags: Tags,
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct SingleExtractedEventWithRaw {
    pub id: String,
    pub timestamp: Timestamptz,
    pub duration: f64,
    pub tags: Tags,
    pub raw: EventData,
    pub tags_reasons: HashMap<String, TagAddReason>,
}

macro_rules! make_thingois {
    (pub enum $name:ident {
        $($r:ident { response: $resp:ty })+
    }) => (
        #[allow(non_camel_case_types)]
        #[derive(TypeScriptify, Serialize)]
        #[serde(tag="type")]
        pub enum $name {
            $( $r { response: $resp }),*
        }
        #[allow(non_snake_case)]
        pub mod Api {
            $(

                pub mod $r {
                    pub use super::super::*;
                    #[allow(non_camel_case_types)]
                    pub type response = DebugRes<Json<ApiResponse<$resp>>>;
                }
            )*
        }
    )
}

use rocket_contrib::json::Json;
type DebugRes<T> = Result<T, rocket::response::Debug<anyhow::Error>>;

make_thingois! {
    pub enum ApiTypesTS {
        time_range {
            response: Vec<SingleExtractedEvent>
        }
        single_event {
            response: Option<SingleExtractedEventWithRaw>
        }
        rule_groups {
            response: Vec<TagRuleGroup>
        }
        update_rule_groups {
            response: ()
        }
    }
}

// https://stackoverflow.com/a/40573155/2639190 :(

// type X = ApiTypes::time_range::response;
