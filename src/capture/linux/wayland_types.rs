use crate::{capture::process::ProcessData, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaylandCaptureArgs {
    // captures from default screen
    /// if true, only capture the focused window.
    /// if false, capture all windows.
    pub only_focused_window: bool,
}

#[cfg(target_os = "linux")]
impl CapturerCreator for WaylandCaptureArgs {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>> {
        // match XDG_CURRENT_DESKTOP
        match std::env::var("XDG_CURRENT_DESKTOP").as_deref() {
            Ok("sway") => {
                super::wayland::init_sway(self.clone()).map(|e| Box::new(e) as Box<dyn Capturer>)
            }
            Ok("Hyprland") => super::wayland::init_hyprland(self.clone())
                .map(|e| Box::new(e) as Box<dyn Capturer>),
            _ => {
                log::warn!("Unknown or unsupported desktop environment");
                anyhow::bail!("Unsupported desktop environment")
            }
        }
    }
}

#[cfg(not(target_os = "linux"))]
impl CapturerCreator for WaylandCaptureArgs {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>> {
        anyhow::bail!("Not on Linux!")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwayEventData {
    pub os_info: util::OsInfo,
    pub ms_since_user_input: u32,
    pub network: Option<NetworkInfo>,
    /// response of swaymsg -t get_tree command
    pub window_tree: serde_json::Value,
    pub processes: Vec<ProcessData>,
}

impl ExtractInfo for SwayEventData {
    fn extract_info(&self) -> Option<Tags> {
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyprlandEventData {
    pub os_info: util::OsInfo,
    pub ms_since_user_input: u32,
    pub network: Option<NetworkInfo>,
    /// response of swaymsg -t get_tree command
    pub window_tree: serde_json::Value,
    pub processes: Vec<ProcessData>,
}
impl ExtractInfo for HyprlandEventData {
    fn extract_info(&self) -> Option<Tags> {
        None
    }
}
