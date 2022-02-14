// MacOS capture types (must be cross-platform)
use super::super::pc_common;
use crate::prelude::*;
use sysinfo::{Process, ProcessExt};
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
    pub windows: Vec<MacOSWindow>,
}

impl ExtractInfo for MacOSEventData {
    fn extract_info(&self) -> Option<Tags> {
        if pc_common::is_idle(self.duration_since_user_input) {
            return None;
        }

        let mut tags = Tags::new();
        
        self.os_info.to_partial_general_software(&mut tags);

        for window in &self.windows {
            let cls = Some((window.process.name.clone(), "".to_owned()));

            let window_title = match window.title {
                Some(ref string) => string.as_str(),
                None => "Unknown",
            };

            tags.extend(pc_common::match_software(
                window_title,
                &cls,
                Some(&window.process.exe),
                None,
                None
            ));
        }

        Some(tags)
    }
}

#[derive(Debug, Default, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct MacOSWindow {
    pub window_id: i32,
    pub title: Option<String>,
    pub process: MacOSProcessData,
}

#[derive(Debug, Default, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct MacOSProcessData {
    pub name: String,
    pub exe: String,
    pub status: String,
}

impl From<&Process> for MacOSProcessData {
    fn from(other: &Process) -> Self {
        MacOSProcessData {
            name: other.name().to_string(),
            exe: other.exe().to_str().unwrap_or_default().to_owned(),
            status: other.status().to_string()
        }
    }
}
