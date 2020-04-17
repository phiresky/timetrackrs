// x11 capture types (must be cross-platform)
use crate::prelude::*;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value as J;
use std::collections::BTreeMap;
use typescript_definitions::TypeScriptify;

#[derive(StructOpt)]
pub struct X11CaptureArgs {
    // captures from default screen, no options really
}

#[cfg(target_os = "linux")]
impl CapturerCreator for X11CaptureArgs {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>> {
        match super::x11::X11Capturer::init() {
            Ok(e) => Ok(Box::new(e)),
            Err(e) => Err(e),
        }
    }
}

#[cfg(not(target_os = "linux"))]
impl CapturerCreator for X11CaptureArgs {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>> {
        anyhow::bail!("Not on Linux!")
    }
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify)]
pub struct X11EventData {
    #[serde(default)]
    pub os_info: util::OsInfo,
    pub desktop_names: Vec<String>,
    pub current_desktop_id: usize,
    pub focused_window: u32,
    pub ms_since_user_input: u32,
    pub ms_until_screensaver: u32,
    pub screensaver_window: u32,
    pub windows: Vec<X11WindowData>,
}
#[derive(Debug, Serialize, Deserialize, TypeScriptify)]
pub struct X11WindowData {
    pub window_id: u32,
    pub geometry: X11WindowGeometry,
    pub process: Option<ProcessData>,
    pub window_properties: BTreeMap<String, J>,
}
#[derive(Debug, Serialize, Deserialize, TypeScriptify)]
pub struct X11WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
#[derive(Debug, Serialize, Deserialize, TypeScriptify)]
pub struct ProcessData {
    pub pid: i32,
    pub name: String,
    pub cmd: Vec<String>,
    pub exe: String,
    pub cwd: String,
    pub memory_kB: i64,
    pub parent: Option<i32>,
    pub status: String,
    pub start_time: DateTime<Utc>,
    pub cpu_usage: Option<f32>, // can be NaN -> null
}
impl X11WindowData {
    fn get_title(&self) -> Option<String> {
        if let Some(J::String(title)) = &self.window_properties.get("_NET_WM_NAME") {
            Some(title.to_string())
        } else {
            None
        }
    }
    fn get_class(&self) -> Option<(String, String)> {
        self.window_properties
            .get("WM_CLASS")
            .and_then(|e| match e {
                J::String(cls) => {
                    let v = split_zero(&cls);
                    Some((v[0].clone(), v[1].clone()))
                }
                _ => None,
            })
    }
}
// "2\u{0}4\u{0}5\u{0}6\u{0}8\u{0}9\u{0}1\u{0}" to array of strings
pub fn split_zero(s: &str) -> Vec<String> {
    let mut vec: Vec<String> = s.split("\0").map(|e| String::from(e)).collect();
    let last = vec.pop().unwrap();
    if last.len() != 0 {
        panic!("not zero terminated");
    }
    return vec;
}

use crate::extract::{properties::ExtractedInfo, ExtractInfo};
impl ExtractInfo for X11EventData {
    fn extract_info(&self) -> Option<ExtractedInfo> {
        use crate::extract::properties::*;
        let x = &self;
        if x.ms_since_user_input > 120 * 1000 {
            return None;
        }
        let mut general = GeneralSoftware {
            hostname: x.os_info.hostname.clone(),
            device_type: if x.os_info.batteries.unwrap_or(0) > 0 {
                SoftwareDeviceType::Laptop
            } else {
                SoftwareDeviceType::Desktop
            },
            device_os: x.os_info.os_type.to_string(),
            identifier: Identifier("".to_string()),
            title: "".to_string(),
            unique_name: "".to_string(),
        };
        let window = x.windows.iter().find(|e| e.window_id == x.focused_window);
        let specific = match window {
            None => SpecificSoftware::Unknown,
            Some(w) => {
                if let Some(window_title) = w.get_title() {
                    let cls = w.get_class();
                    super::pc_common::match_from_title(
                        &mut general,
                        &window_title,
                        &cls,
                        w.process.as_ref().map(|p| p.exe.as_ref()),
                    )
                } else {
                    SpecificSoftware::Unknown
                }
            }
        };
        Some(ExtractedInfo::UseDevice { general, specific })
    }
}
