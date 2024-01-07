pub mod linux;
pub mod macos;
pub mod pc_common;
pub mod process;
pub mod windows;
use std::time::Duration;

use futures::never::Never;

use crate::{capture::linux::wayland_types::WaylandCaptureArgs, prelude::*};

#[enum_dispatch]
#[derive(Debug, Serialize, Deserialize)]
pub enum CaptureArgs {
    /// Capture open window information from a (linux) X11 server
    X11(X11CaptureArgs),
    Wayland(WaylandCaptureArgs),
    Windows(WindowsCaptureArgs),
    MacOS(MacOSCaptureArgs),
    /// Capture window information using the default for the current system
    NativeDefault(NativeDefaultArgs),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NativeDefaultArgs {}

fn default_capture_args() -> anyhow::Result<CaptureArgs> {
    #[cfg(target_os = "linux")]
    {
        let session = std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "".to_string());
        return Ok(match session.as_ref() {
            "wayland" => CaptureArgs::Wayland(WaylandCaptureArgs {
                only_focused_window: false,
            }),
            "x11" => CaptureArgs::X11(X11CaptureArgs {
                only_focused_window: false,
            }),
            _ => {
                anyhow::bail!("Unknown XDG_SESSION_TYPE: {}", session);
            }
        });
    }
    #[cfg(target_os = "windows")]
    return CaptureArgs::Windows(WindowsCaptureArgs {});

    #[cfg(target_os = "macos")]
    return CaptureArgs::MacOS(MacOSCaptureArgs {});
}

impl CapturerCreator for NativeDefaultArgs {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>> {
        default_capture_args()?.create_capturer()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CaptureConfig {
    pub interval: Duration,
    pub args: CaptureArgs,
}

#[enum_dispatch(CaptureArgs)]
pub trait CapturerCreator {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>>;
}

#[async_trait]
pub trait Capturer: Send {
    async fn capture(&mut self) -> anyhow::Result<EventData>;
}

pub async fn capture_loop(db: DatyBasy, config: CaptureConfig) -> anyhow::Result<Never> {
    let CaptureConfig { args, interval: _ } = config;
    let mut c = args
        .create_capturer()
        .with_context(|| format!("Could not create capturer from {:?}", &args))?;

    let idgen = crate::libxid::new_generator();

    let mut interval = tokio::time::interval(config.interval);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        log::info!("sleeping {}s", config.interval.as_secs());
        interval.tick().await;

        match c.capture().await {
            Ok(data) => {
                let act = CreateNewDbEvent {
                    id: idgen.new_id().unwrap().encode(),
                    timestamp: Utc::now(),
                    duration_ms: config.interval.as_millis() as i64,
                    data,
                };
                let ins: NewDbEvent = act.try_into()?;

                db.insert_events_if_needed(vec![ins])
                    .await
                    .context("Could not insert captured event")?;
            }
            Err(e) => {
                log::error!("Could not capture event: {}", e);
            }
        }
    }
}
