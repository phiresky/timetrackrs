// windows capture types (must be cross-platform)
use crate::prelude::*;

#[derive(Debug)]
pub struct WindowsCaptureArgs {}

#[cfg(windows)]
impl CapturerCreator for WindowsCaptureArgs {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>> {
        match super::winwins::WindowsCapturer::init() {
            Ok(e) => Ok(Box::new(e)),
            Err(e) => Err(e),
        }
    }
}
#[cfg(not(windows))]
impl CapturerCreator for WindowsCaptureArgs {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>> {
        anyhow::bail!("Not on Linux!")
    }
}
#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct WindowsEventData {
    pub os_info: util::OsInfo,
    pub focused_window: Option<i64>,
    pub windows: Vec<WindowsWindow>,
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct WindowsWindow {
    pub window_id: i64,
    pub title: String,
    pub exe: Option<String>,
    pub wclass: String,
}

use crate::extract::ExtractInfo;
impl ExtractInfo for WindowsEventData {
    fn extract_info(&self) -> Option<tags::Tags> {
        let tags = tags::Tags::new();
        Some(tags)
        /*use crate::extract::properties::*;
            let x = &self;
            /*if x.ms_since_user_input > 120 * 1000 {
                return None;
            }*/
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
                opened_filepath: None,
            };
            let window = x
                .windows
                .iter()
                .find(|e| Some(e.window_id) == x.focused_window);
            let specific = match window {
                None => SpecificSoftware::Unknown,
                Some(w) => {
                    let cls = Some((w.wclass.clone(), "".to_string()));
                    super::pc_common::match_software(
                        &mut general,
                        &w.title,
                        &cls,
                        w.exe.as_ref().map(|e| e.as_ref()),
                        None, // TODO
                        None, // TODO
                    )
                }
            };
            Some(ExtractedInfo::InteractWithDevice { general, specific })
        }*/
    }
}
