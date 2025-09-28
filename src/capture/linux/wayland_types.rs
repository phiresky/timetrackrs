use crate::{capture::process::ProcessData, prelude::*};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

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
                // anyhow::bail!("Unsupported desktop environment")
                super::wayland::WaylandForeignTopLevelManagerCapturer::new()
                    .map(|e| Box::new(e) as Box<dyn Capturer>)
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
        use std::time::Duration;
        let mut tags = Tags::new();

        if super::super::pc_common::is_idle(Duration::from_millis(self.ms_since_user_input as u64))
        {
            return None;
        }

        self.os_info.to_partial_general_software(&mut tags);

        if let Some(NetworkInfo { wifi: Some(wifi) }) = &self.network {
            tags.add("connected-wifi", &wifi.ssid);
        }

        // Find the focused window in the Sway tree
        if let Some(focused_window) = find_focused_sway_window(&self.window_tree) {
            if let Some(pid) = focused_window.get("pid").and_then(|p| p.as_u64()) {
                // Find the corresponding process data
                if let Some(process) = self.processes.iter().find(|p| p.pid == pid as i32) {
                    let window_title = focused_window
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("");

                    // Try to get class from window properties (X11 windows) or app_id (Wayland windows)
                    let window_class =
                        if let Some(window_props) = focused_window.get("window_properties") {
                            if let (Some(class), Some(instance)) = (
                                window_props.get("class").and_then(|c| c.as_str()),
                                window_props.get("instance").and_then(|i| i.as_str()),
                            ) {
                                Some((class.to_string(), instance.to_string()))
                            } else {
                                None
                            }
                        } else {
                            focused_window
                                .get("app_id")
                                .and_then(|id| id.as_str())
                                .map(|app_id| (app_id.to_string(), "".to_string()))
                        };

                    tags.extend(super::super::pc_common::match_software(
                        window_title,
                        &window_class,
                        process.exe.as_deref(),
                        process.cwd.as_deref(),
                        Some(&process.cmd),
                    ));
                }
            }
        }

        Some(tags)
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
        use std::time::Duration;
        let mut tags = Tags::new();

        if super::super::pc_common::is_idle(Duration::from_millis(self.ms_since_user_input as u64))
        {
            return None;
        }

        self.os_info.to_partial_general_software(&mut tags);

        if let Some(NetworkInfo { wifi: Some(wifi) }) = &self.network {
            tags.add("connected-wifi", &wifi.ssid);
        }

        // Find the focused window in Hyprland (the one with focusHistoryID: 0)
        if let Some(windows) = self.window_tree.as_array() {
            if let Some(focused_window) = windows
                .iter()
                .find(|w| w.get("focusHistoryID").and_then(|id| id.as_u64()) == Some(0))
            {
                if let Some(pid) = focused_window.get("pid").and_then(|p| p.as_u64()) {
                    // Find the corresponding process data
                    if let Some(process) = self.processes.iter().find(|p| p.pid == pid as i32) {
                        let window_title = focused_window
                            .get("title")
                            .and_then(|t| t.as_str())
                            .unwrap_or("");

                        // Use class and initialClass for window class
                        let window_class = if let (Some(class), Some(initial_class)) = (
                            focused_window.get("class").and_then(|c| c.as_str()),
                            focused_window
                                .get("initialClass")
                                .and_then(|ic| ic.as_str()),
                        ) {
                            Some((class.to_string(), initial_class.to_string()))
                        } else {
                            focused_window
                                .get("class")
                                .and_then(|c| c.as_str())
                                .map(|class| (class.to_string(), "".to_string()))
                        };

                        tags.extend(super::super::pc_common::match_software(
                            window_title,
                            &window_class,
                            process.exe.as_deref(),
                            process.cwd.as_deref(),
                            Some(&process.cmd),
                        ));

                        // Add Hyprland-specific tags
                        if let Some(workspace) = focused_window.get("workspace") {
                            if let Some(workspace_name) =
                                workspace.get("name").and_then(|n| n.as_str())
                            {
                                tags.add("window-manager-workspace", workspace_name);
                            }
                        }
                    }
                }
            }
        }

        Some(tags)
    }
}

fn find_focused_sway_window(node: &JsonValue) -> Option<&JsonValue> {
    // Check if this node is focused
    if let Some(focused) = node.get("focused") {
        if focused.as_bool() == Some(true) {
            return Some(node);
        }
    }

    // Recursively search in child nodes
    if let Some(nodes) = node.get("nodes").and_then(|n| n.as_array()) {
        for child in nodes {
            if let Some(found) = find_focused_sway_window(child) {
                return Some(found);
            }
        }
    }

    // Also search in floating nodes
    if let Some(floating_nodes) = node.get("floating_nodes").and_then(|n| n.as_array()) {
        for child in floating_nodes {
            if let Some(found) = find_focused_sway_window(child) {
                return Some(found);
            }
        }
    }

    None
}
