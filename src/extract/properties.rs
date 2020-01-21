/* properties

property-category.property-name: (property-type, )
different property types:
- (identifier): identifier that can be looked up elsewhere
- (text, expected): "expected" is the decimal order of magnitude of how many values of this might be expected to appear for one user in a given year.

# pc stuff
*/
use serde::Serialize;
use typescript_definitions::TypeScriptify;

#[derive(Default, Serialize, TypeScriptify)]
pub struct ExtractedInfo {
    pub event_id: String, // event to refer back to the raw event
    pub software_development: Option<SoftwareDevelopment>,
    pub shell: Option<Shell>,
    pub web_browser: Option<WebBrowser>,
    pub media_player: Option<MediaPlayer>,
    pub software: Option<Software>,
    pub physical_activity: Option<PhysicalActivity>,
}
#[derive(Serialize, TypeScriptify)]
pub struct Identifier(pub String);
type Text10 = String;
type Text100 = String;
type Text1000 = String;
type Text10000 = String;
type Text100000 = String;

#[derive(Serialize, TypeScriptify)]
pub struct SoftwareDevelopment {
    pub project_path: Option<Text100>,
    pub file_path: Text1000,
}
#[derive(Serialize, TypeScriptify)]
pub struct Shell {
    pub cwd: Text1000,
    pub cmd: Text10000,
    pub zsh_histdb_session_id: Identifier,
}
#[derive(Serialize, TypeScriptify)]
pub struct WebBrowser {
    pub url: Text10000,
    // TODO: needs public suffix list
    // pub main_domain: Text1000, // main domain name (e.g. old.reddit.com -> reddit.com)
    pub origin: Text1000,  // full origin (https://example.com) of browsed url
    pub service: Text1000, // name of the webservice being used, based on url. e.g. "Reddit" or "GMail"
}
#[derive(Serialize, TypeScriptify)]
pub enum MediaType {
    Audio,
    Video,
}
#[derive(Serialize, TypeScriptify)]
pub struct MediaPlayer {
    pub media_filename: Text1000,
    pub media_type: MediaType,
    pub media_name: Text1000, // (e.g. movie title)
}
#[derive(Serialize, TypeScriptify)]
pub enum SoftwareDeviceType {
    Desktop,
    Laptop,
    Smartphone,
    Tablet,
}
#[derive(Serialize, TypeScriptify)]
pub struct Software {
    pub hostname: Text100,
    pub device_type: SoftwareDeviceType,
    pub device_os: Text10,
    pub title: Text10000,
    pub identifier: Identifier, // unique identifier for software package e.g. android:com.package.id or exe:/binary/path
    pub unique_name: Text100, // name of software that should be globally unique and generally recognizable (e.g. "Firefox")
}
#[derive(Serialize, TypeScriptify)]
pub struct PhysicalActivity {
    pub activity_type: Text100, //  (walking|biking|etc.)
}
