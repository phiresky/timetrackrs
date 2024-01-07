// x11 capture types (must be cross-platform)
use crate::{capture::process::ProcessData, prelude::*};
use serde::{Deserialize, Serialize};
use serde_json::Value as J;
use std::collections::BTreeMap;
use typescript_definitions::TypeScriptify;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct X11CaptureArgs {
    // captures from default screen
    /// if true, only capture the focused window.
    /// if false, capture all windows.
    pub only_focused_window: bool,
}

#[cfg(target_os = "linux")]
impl CapturerCreator for X11CaptureArgs {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>> {
        super::x11::init(self.clone()).map(|e| Box::new(e) as Box<dyn Capturer>)
    }
}

#[cfg(not(target_os = "linux"))]
impl CapturerCreator for X11CaptureArgs {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>> {
        anyhow::bail!("Not on Linux!")
    }
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct X11EventData {
    #[serde(default)]
    pub os_info: util::OsInfo,
    pub desktop_names: Vec<String>,
    pub current_desktop_id: usize,
    pub focused_window: u32,
    pub ms_since_user_input: u32,
    pub ms_until_screensaver: u32,
    pub screensaver_window: u32,
    pub network: Option<NetworkInfo>,
    pub windows: Vec<X11WindowData>,
}
#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct X11WindowData {
    pub window_id: u32,
    pub geometry: X11WindowGeometry,
    pub process: Option<ProcessData>,
    pub window_properties: BTreeMap<String, J>,
}
#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct X11WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct WifiInterface {
    /// Interface essid
    pub ssid: String,
    /// Interface MAC address
    pub mac: String,
    /// Interface name (u8, String)
    pub name: String,
    /// Interface transmit power level in signed mBm units.
    pub power: u32,
    /// Signal strength average (i8, dBm)
    pub average_signal: i8,
    /// Station bssid (u8)
    pub bssid: String,
    /// Time since the station is last connected in seconds (u32)
    pub connected_time: u32,
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct NetworkInfo {
    pub wifi: Option<WifiInterface>,
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
                    let v = split_zero(cls);
                    Some((v[0].clone(), v[1].clone()))
                }
                _ => None,
            })
    }
}
// "2\u{0}4\u{0}5\u{0}6\u{0}8\u{0}9\u{0}1\u{0}" to array of strings
pub fn split_zero(s: &str) -> Vec<String> {
    let mut vec: Vec<String> = s.split('\0').map(String::from).collect();
    if vec.last().map(|e| e.is_empty()).unwrap_or(false) {
        // there seems to be an inconsistency:
        // the list in WM_CLASS is zero-terminated, as is the list in _NET_DESKTOP_NAMES on i3
        // but in bspwm it is not zero-terminated
        // https://github.com/phiresky/timetrackrs/issues/12
        vec.pop().unwrap();
    }
    vec
}

use crate::extract::ExtractInfo;
impl ExtractInfo for X11EventData {
    fn extract_info(&self) -> Option<Tags> {
        let mut tags = Tags::new();
        let x = &self;
        if super::super::pc_common::is_idle(Duration::from_millis(x.ms_since_user_input as u64)) {
            return None;
        }
        x.os_info.to_partial_general_software(&mut tags);
        if let Some(NetworkInfo { wifi: Some(wifi) }) = &x.network {
            tags.add("connected-wifi", &wifi.ssid);
        }
        let window = x.windows.iter().find(|e| e.window_id == x.focused_window);
        match window {
            None => (),
            Some(w) => {
                if let Some(window_title) = w.get_title() {
                    let cls = w.get_class();
                    tags.extend(super::super::pc_common::match_software(
                        &window_title,
                        &cls,
                        w.process
                            .as_ref()
                            .and_then(|p| p.exe.as_ref().map(|e| e.as_ref())),
                        w.process
                            .as_ref()
                            .and_then(|p| p.cwd.as_ref().map(|e| e.as_ref())),
                        w.process.as_ref().map(|p| p.cmd.as_ref()),
                    ));
                }
            }
        };
        Some(tags)
    }
}
