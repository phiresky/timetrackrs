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
    pub on_screen_windows: Option<Vec<usize>>,
    pub windows: Vec<MacOSWindow>,
}

impl ExtractInfo for MacOSEventData {
    fn extract_info(&self) -> Option<Tags> {
        if self.on_screen_windows.is_none() {
            return None;
        }

        if pc_common::is_idle(self.duration_since_user_input) {
            return None;
        }

        let mut tags = Tags::new();
        
        self.os_info.to_partial_general_software(&mut tags);

        for i in self.on_screen_windows.as_ref().unwrap() {
            let window = &self.windows[*i];
            log::debug!("Yep WinCock {:?}", window);
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
    pub process: Arc<MacOSProcessData>,
}

#[derive(Debug, Default, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct MacOSProcessData {
    pub name: String,
    pub exe: String,
    pub status: String,
}
