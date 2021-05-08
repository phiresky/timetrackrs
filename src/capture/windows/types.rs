// windows capture types (must be cross-platform)
use crate::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
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
        anyhow::bail!("Not on Windows!")
    }
}
#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct WindowsEventData {
    pub os_info: util::OsInfo,
    pub focused_window: Option<i64>,
    pub windows: Vec<WindowsWindow>,
    pub duration_since_user_input: std::time::Duration,
    pub wifi: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct WindowsWindow {
    pub window_id: i64,
    pub process_id: Option<i64>,
    pub title: String,
    pub wclass: String,
    pub exe: Option<String>,
    pub process_started: Option<DateTime<Utc>>,
    pub command_line: Option<Vec<String>>,
}

use crate::extract::ExtractInfo;
impl ExtractInfo for WindowsEventData {
    fn extract_info(&self) -> Option<tags::Tags> {
        let mut tags = tags::Tags::new();
        if super::super::pc_common::is_idle(self.duration_since_user_input) {
            return None;
        }
        self.os_info.to_partial_general_software(&mut tags);
        if let Some(str) = &self.wifi {
            tags.add("connected-wifi", str);
        }
        let window = self
            .windows
            .iter()
            .find(|e| Some(e.window_id) == self.focused_window);
        match window {
            None => (),
            Some(w) => {
                let cls = Some((w.wclass.clone(), "".to_string()));
                tags.extend(super::super::pc_common::match_software(
                    &w.title,
                    &cls,
                    w.exe.as_ref().map(|e| e.as_ref()),
                    None, // TODO
                    w.command_line.as_ref().map(|e| &e[..]),
                ));
            }
        };
        Some(tags)
    }
}
