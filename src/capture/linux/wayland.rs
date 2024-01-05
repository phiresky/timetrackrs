use tokio::process::Command;

use super::wayland_types::WaylandCaptureArgs;
use crate::prelude::*;

// refs:
// https://github.com/ActivityWatch/aw-watcher-window-wayland

// Right now there is no standard way to get the list of windows on wayland. There's a proposal called wlr_foreign_toplevel_management_v1
// that works for a few WMs but not the major ones (Gnome, KDE)
// so until that shit is fixed, I'll just use the sway API since that's what I care about

// swaymsg -t get_tree

pub fn init(_options: WaylandCaptureArgs) -> anyhow::Result<SwayCapturer> {
    Ok(SwayCapturer)
}
pub struct SwayCapturer;

#[async_trait]
impl Capturer for SwayCapturer {
    async fn capture(&mut self) -> anyhow::Result<EventData> {
        let res = Command::new("swaymsg")
            .arg("-t")
            .arg("get_tree")
            .output()
            .await?;
        if !res.status.success() {
            anyhow::bail!(
                "Could not run swaymsg: {} {}",
                String::from_utf8_lossy(&res.stdout),
                String::from_utf8_lossy(&res.stderr)
            )
        }
        let parsed: serde_json::Value = serde_json::from_slice(&res.stdout)?;
        Ok(EventData::sway_v1(SwayEventData { tree: parsed }))
    }
}
