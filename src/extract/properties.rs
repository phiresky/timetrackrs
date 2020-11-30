use diesel::SqliteConnection;

use crate::prelude::*;

#[derive(Serialize, TypeScriptify)]
#[serde(tag = "type")]
pub enum ExtractedInfo {
    InteractWithDevice {
        general: GeneralSoftware,
        specific: SpecificSoftware,
    },
    PhysicalActivity {
        activity_type: Text100Choices, //  (walking|biking|etc.)
    },
}
#[derive(Serialize, TypeScriptify)]
#[serde(tag = "type")]
pub enum SpecificSoftware {
    DeviceStateChange {
        change: DeviceStateChange,
    },
    WebBrowser {
        url: Option<Text10000Choices>,
        // TODO: needs public suffix list
        // pub main_domain: Text1000, // main domain name (e.g. old.reddit.com -> reddit.com)
        domain: Option<Text1000Choices>, // domain (e.g. web.telegram.org) of browsed url
    },
    Shell {
        cwd: Text1000Choices,
        cmd: Text10000Choices,
        zsh_histdb_session_id: Identifier,
    },
    MediaPlayer {
        media_filename: Text1000Choices,
        media_type: MediaType,
        media_name: Text1000Choices, // (e.g. movie title)
    },
    SoftwareDevelopment {
        project_path: Option<Text100Choices>,
        file_path: Text1000Choices,
    },
    Unknown,
}

#[derive(Serialize, TypeScriptify)]
/** - some generic identifier that can be looked up elsewhere. i.e. something that should be unique within the corresponding scope of the surrounding object */
pub struct Identifier(pub String);

/**
the number is the decimal order of magnitude of how many values of this might be expected to appear for one user in a given year.

e.g. a user will probably perform 10 - 100 different physical activities in a given year, so the type used should be Text100Choices

this info might be used in the future for heuristics of what to display

or it might turn out to be useless / estimatable on demand
 */
type Text10Choices = String;
type Text100Choices = String;
type Text1000Choices = String;
type Text10000Choices = String;
type Text100000Choices = String;

#[derive(Serialize, TypeScriptify)]
pub enum MediaType {
    Audio,
    Video,
}
#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub enum SoftwareDeviceType {
    Desktop,
    Laptop,
    Smartphone,
    Tablet,
}
#[derive(Serialize, TypeScriptify)]
pub struct GeneralSoftware {
    pub hostname: Text100Choices,
    pub device_type: SoftwareDeviceType,
    pub device_os: Text10Choices,
    pub title: Text10000Choices,
    // unique identifier for software package e.g. android:com.package.id or exe:/binary/path
    // not directly using exe path since some platforms (android) don't really have that
    pub identifier: Identifier,
    pub unique_name: Text100Choices, // name of software that should be globally unique and generally recognizable (e.g. "Firefox")
    pub opened_filepath: Option<Text10000Choices>,
}

// a URI in the format protocol://path/... of the specific thing that was done / accessed
//
// for device usage, this should NOT be what software / method was used, but rather the thing that was accessed (e.g. not "Google Chrome" but "reddit")
//
// the slashes should indicate some kind of structure, where later values are more specific
// then aggregation can automatically be done by merging activities with common path prefixes
//
// for example:
//
// URLs:
// https://reddit.com/r/subreddit/postid
// means the user looked at reddit, at a specific subreddit, at a specific post
//
//
// files:
// file://hostname/home/username/studying/maths101/slide01.pdf
// means the user looked at a specific file
// the file is in directory "/.../studying" so it can probably be aggregated into the "studying" category
//
// activities:
//
// activity:biking

#[derive(Serialize, TypeScriptify)]
pub enum DeviceStateChange {
    PowerOn,
    PowerOff,
    Sleep,
    Wakeup,
}

#[derive(Serialize, TypeScriptify)]
pub struct EnrichedExtractedInfo {
    tags: Vec<String>,
    info: ExtractedInfo,
}

pub fn enrich_extracted_info(db: &mut SqliteConnection, o: ExtractedInfo) -> EnrichedExtractedInfo {
    EnrichedExtractedInfo {
        tags: tags::get_tags(db, &o).into_iter().collect(),
        info: o,
    }
}
