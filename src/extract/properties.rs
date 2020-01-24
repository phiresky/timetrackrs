/* properties

property-category.property-name: (property-type, )
different property types:
- (identifier): identifier that can be looked up elsewhere
- (text, expected): "expected" is the decimal order of magnitude of how many values of this might be expected to appear for one user in a given year.

# pc stuff
*/
use crate::prelude::*;

#[derive(Serialize, TypeScriptify)]
#[serde(tag = "type")]
pub enum ExtractedInfo {
    UseDevice {
        general: GeneralSoftware,
        specific: SpecificSoftware,
    },
    PhysicalActivity {
        activity_type: Text100, //  (walking|biking|etc.)
    },
}
#[derive(Serialize, TypeScriptify)]
#[serde(tag = "type")]
pub enum SpecificSoftware {
    WebBrowser {
        url: Text10000,
        // TODO: needs public suffix list
        // pub main_domain: Text1000, // main domain name (e.g. old.reddit.com -> reddit.com)
        origin: Text1000,  // full origin (https://example.com) of browsed url
        service: Text1000, // name of the webservice being used, based on url. e.g. "Reddit" or "GMail"
    },
    Shell {
        cwd: Text1000,
        cmd: Text10000,
        zsh_histdb_session_id: Identifier,
    },
    MediaPlayer {
        media_filename: Text1000,
        media_type: MediaType,
        media_name: Text1000, // (e.g. movie title)
    },
    SoftwareDevelopment {
        project_path: Option<Text100>,
        file_path: Text1000,
    },
    Unknown,
}
#[derive(Serialize, TypeScriptify)]
pub struct Identifier(pub String);
type Text10 = String;
type Text100 = String;
type Text1000 = String;
type Text10000 = String;
type Text100000 = String;

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
    pub hostname: Text100,
    pub device_type: SoftwareDeviceType,
    pub device_os: Text10,
    pub title: Text10000,
    pub identifier: Identifier, // unique identifier for software package e.g. android:com.package.id or exe:/binary/path
    pub unique_name: Text100, // name of software that should be globally unique and generally recognizable (e.g. "Firefox")
}
