pub mod linux;
pub mod pc_common;
pub mod windows;
pub mod macos;

use std::time::Duration;

use futures::never::Never;

use crate::prelude::*;

#[enum_dispatch]
#[derive(Debug, Serialize, Deserialize)]
pub enum CaptureArgs {
    /// Capture open window information from a (linux) X11 server
    X11(X11CaptureArgs),
    Windows(WindowsCaptureArgs),
    MacOS(MacOSCaptureArgs),
    /// Capture window information using the default for the current system
    NativeDefault(NativeDefaultArgs),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NativeDefaultArgs {}

fn default_capture_args() -> CaptureArgs {
    #[cfg(target_os = "linux")]
    return CaptureArgs::X11(X11CaptureArgs {
        only_focused_window: false,
    });
    #[cfg(target_os = "windows")]
    return CaptureArgs::Windows(WindowsCaptureArgs {});
    
    #[cfg(target_os = "macos")]
    return CaptureArgs::MacOS(MacOSCaptureArgs{});
}

impl CapturerCreator for NativeDefaultArgs {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>> {
        default_capture_args().create_capturer()
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

pub trait Capturer: Send {
    fn capture(&mut self) -> anyhow::Result<EventData>;
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

        let data = c.capture()?;
        let act = CreateNewDbEvent {
            id: idgen.new_id().unwrap().encode(),
            timestamp: Utc::now(),
            duration_ms: config.interval.as_millis() as i64,
            data,
        };
        let ins: NewDbEvent = act.try_into()?;
        
        if cfg!(feature = "graphql"){
            use crate::graphql;
            // ToDo: Instead of cloning, share the data as reference.
            graphql::insert_tracker_event(ins.clone()).await?;
        }
        
        db.insert_events_if_needed(vec![ins])
            .await
            .context("Could not insert captured event")?;
    }
}
