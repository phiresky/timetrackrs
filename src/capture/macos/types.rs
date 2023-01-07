// MacOS capture types (must be cross-platform)
use super::super::pc_common;
use crate::prelude::*;

use std::{sync::Arc, time::Duration};

#[derive(Debug, Serialize, Deserialize)]
pub struct MacOSCaptureArgs {}

#[cfg(target_os = "macos")]
impl CapturerCreator for MacOSCaptureArgs {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>> {
        Ok(Box::new(super::appkit::MacOSCapturer::init()))
    }
}

#[cfg(not(target_os = "macos"))]
impl CapturerCreator for MacOSCaptureArgs {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>> {
        anyhow::bail!("Not on MacOS!")
    }
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct MacOSEventData {
    #[serde(default)]
    pub os_info: util::OsInfo,
    pub duration_since_user_input: Duration,
    pub focused_window: Option<i32>,
    pub windows: Vec<MacOSWindow>,
}

impl ExtractInfo for MacOSEventData {
    fn extract_info(&self) -> Option<Tags> {
        let mut tags = Tags::new();

        if pc_common::is_idle(self.duration_since_user_input) {
            return None;
        }

        self.os_info.to_partial_general_software(&mut tags);

        if let Some(focused_window) = self.focused_window {
            if let Some(window) = self.windows.iter().find(|w| w.window_id == focused_window) {
                if let Some(process) = &window.process {
                    let cls = Some((process.name.clone(), "".to_owned()));

                    let window_title = match window.title {
                        Some(ref string) => string.as_str(),
                        None => "Unknown",
                    };

                    tags.extend(pc_common::match_software(
                        window_title,
                        &cls,
                        Some(&process.exe),
                        None,
                        Some(&process.cmd),
                    ));
                }
            }
        }

        Some(tags)
    }
}

#[derive(Debug, Default, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct MacOSWindow {
    pub window_id: i32,
    pub title: Option<String>,
    pub process: Option<Arc<MacOSProcessData>>,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct MacOSProcessData {
    pub pid: i32,
    pub name: String,
    pub bundle: Option<String>,
    pub cmd: Vec<String>,
    pub exe: String,
    pub cwd: String,

    pub memory_kB: i64,
    pub parent: Option<i32>,
    pub status: String,
    pub start_time: DateTime<Utc>,
    pub cpu_usage: Option<f32>, // can be NaN -> null
}
