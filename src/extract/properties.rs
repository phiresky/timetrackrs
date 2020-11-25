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
        origin: Option<Text1000Choices>, // full origin (https://example.com) of browsed url
        service: Option<Text1000Choices>, // name of the webservice being used, based on url. e.g. "Reddit" or "GMail"
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
#[derive(Serialize, TypeScriptify)]
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
    pub identifier: Identifier, // unique identifier for software package e.g. android:com.package.id or exe:/binary/path
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

pub fn get_tags(e: &ExtractedInfo) -> Vec<String> {
    let mut tags = Vec::new();
    if let ExtractedInfo::InteractWithDevice {
        specific: SpecificSoftware::WebBrowser { url: Some(url), .. },
        ..
    } = e
    {
        tags.push(format!("url:/{}", url.to_string()));
    }
    if let ExtractedInfo::InteractWithDevice {
        general:
            GeneralSoftware {
                opened_filepath: Some(path),
                hostname,
                ..
            },
        ..
    } = e
    {
        tags.push(format!("file://{}{}", hostname, path));
    }
    tags
}

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

impl From<ExtractedInfo> for EnrichedExtractedInfo {
    fn from(o: ExtractedInfo) -> EnrichedExtractedInfo {
        EnrichedExtractedInfo {
            tags: get_tags(&o),
            info: o,
        }
    }
}
