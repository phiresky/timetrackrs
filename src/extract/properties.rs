/* properties

property-category.property-name: (property-type, )
different property types:
- (identifier): identifier that can be looked up elsewhere
- (text, expected): "expected" is the decimal order of magnitude of how many values of this might be expected to appear for one user in a given year.

# pc stuff
*/
use crate::capture::x11::{split_zero, X11WindowData};
use crate::capture::CapturedData;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use serde_json::{json, Value as J};
use std::collections::HashMap;
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
pub struct Identifier(String);
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

lazy_static! {
    static ref FORMATTED_TITLE_MATCH: Regex = Regex::new(r#"🛤([a-z]{2,5})🠚(.*)🠘"#).unwrap();
    static ref FORMATTED_TITLE_SPLIT: Regex = Regex::new("🙰").unwrap();
    static ref FORMATTED_TITLE_KV: Regex = Regex::new("^([a-z0-9]+)=(.*)$").unwrap();
    static ref SH_JSON_TITLE: Regex = Regex::new(r#"\{".*[^\\]"}"#).unwrap();
    static ref UNINTERESTING_BINARY: Regex = Regex::new(r#"/electron\d*$"#).unwrap();
    static ref BROWSER_BINARY: Regex = Regex::new(r#"/(firefox|google-chrome|chromium)$"#).unwrap();
    static ref URL: Regex =
        Regex::new(r#"(?i)https?://(-\.)?([^\s/?\.#-]+\.?)+(/[^\s]*)?"#).unwrap();
        // Regex::new(r#"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)"#).unwrap();
}
fn match_from_title(window: &X11WindowData, info: &mut ExtractedInfo) {
    if let Some(J::String(title)) = &window.window_properties.get("_NET_WM_NAME") {
        if let Some(sw) = &mut info.software {
            sw.title = title.clone();
            if let Some(p) = &window.process {
                sw.identifier = Identifier(p.exe.to_string());
                sw.unique_name = p.exe.to_string();
                if UNINTERESTING_BINARY.is_match(&p.exe) {
                    if let Some(J::String(cls)) = window.window_properties.get("WM_CLASS") {
                        let v = split_zero(cls);
                        sw.identifier = Identifier(format!("WM_CLASS:{}.{}", v[0], v[1]));
                    }
                }
            }
        };
        // match strictly formatted stuff in title:
        // 🛤sd🠚proj=/project/name🙰file=file/name🠘
        if let Some(cap) = FORMATTED_TITLE_MATCH.captures(title) {
            let category = cap.get(1).unwrap().as_str();
            let kv = {
                let mut kv = HashMap::new();
                let values = cap.get(2).unwrap().as_str();
                for kvs in FORMATTED_TITLE_SPLIT.split(values) {
                    let c = FORMATTED_TITLE_KV.captures(kvs).unwrap();
                    let k = c.get(1).unwrap().as_str().to_string();
                    let v = c.get(2).unwrap().as_str().to_string();
                    kv.insert(k, v);
                }
                kv
            };
            match category {
                "sd" => {
                    info.software_development = Some(SoftwareDevelopment {
                        project_path: kv.get("proj").or(kv.get("project")).map(|e| e.to_string()),
                        file_path: kv.get("file").unwrap().to_string(),
                    })
                }
                _ => {
                    println!("unknown category in title info: {}", category);
                }
            }
        }
        if let Some(m) = SH_JSON_TITLE.find(title) {
            if let Ok(J::Object(o)) = serde_json::from_str(m.as_str()) {
                if let (
                    Some(J::String(cwd)),
                    Some(J::Number(histdb)),
                    Some(J::String(usr)),
                    Some(J::String(cmd)),
                ) = (o.get("cwd"), o.get("histdb"), o.get("usr"), o.get("cmd"))
                {
                    info.shell = Some(Shell {
                        cwd: cwd.to_string(),
                        cmd: cmd.to_string(),
                        zsh_histdb_session_id: Identifier(histdb.to_string()),
                    })
                }
            }
        }
        if let Some(p) = &window.process {
            if BROWSER_BINARY.is_match(&p.exe) {
                if let Some(cap) = URL.find(&title) {
                    if let Ok(url) = url::Url::parse(cap.as_str()) {
                        info.web_browser = Some(WebBrowser {
                            url: cap.as_str().to_string(),
                            origin: url.origin().ascii_serialization(),
                            service: url.domain().unwrap().to_string(),
                        });
                    }
                }
            }
        }
    }
}

// if returns None, event is discarded
pub fn extract_info(event_id: String, data: &CapturedData) -> Option<ExtractedInfo> {
    match data {
        CapturedData::x11(x) => {
            if x.ms_since_user_input > 120 * 1000 {
                return None;
            }
            let window = x.windows.iter().find(|e| e.window_id == x.focused_window);
            let mut info = ExtractedInfo {
                event_id,
                ..Default::default()
            }; /* Software {
                   pub device: Text100,
                   pub device_type: SoftwareDeviceType,
                   pub device_os: Text10,
                   pub title: Text10000,
                   pub identifier: Identifier, // unique identifier for software package e.g. android:com.package.id or pc:hostname/binary
                   pub unique_name: Text100, // name of software that should be globally unique and generally recognizable (e.g. "Firefox")
               }*/
            info.software = Some(Software {
                hostname: x.os_info.hostname.clone(),
                device_type: if x.os_info.batteries > 0 {
                    SoftwareDeviceType::Laptop
                } else {
                    SoftwareDeviceType::Desktop
                },
                device_os: x.os_info.os_type.to_string(),
                identifier: Identifier("".to_string()),
                title: "".to_string(),
                unique_name: "".to_string(),
            });
            match window {
                None => return None,
                Some(w) => match_from_title(w, &mut info),
            };
            Some(info)
        }
        CapturedData::app_usage(x) => {
            if x.act_type == crate::import::app_usage_sqlite::UseType::UseApp {
                let pkg_name = x.pkg_name.as_deref().unwrap_or("").to_string();
                Some(ExtractedInfo {
                    software: Some(Software {
                        hostname: x.device_name.clone(),
                        device_type: SoftwareDeviceType::Smartphone,
                        device_os: "Android".to_string(),
                        title: pkg_name.clone(),
                        identifier: Identifier(format!(
                            "android:{}",
                            x.pkg_pkg.as_deref().unwrap_or("??").to_string()
                        )),
                        unique_name: pkg_name.clone(),
                    }),
                    ..Default::default()
                })
            } else {
                None
            }
        }
    }
}
