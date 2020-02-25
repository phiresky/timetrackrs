pub mod pc_common;
#[cfg(windows)]
pub mod winwins;
pub mod winwins_types;
#[cfg(target_os="linux")]
pub mod x11;
pub mod x11_types;
use crate::prelude::*;

#[enum_dispatch]
#[derive(StructOpt)]
#[structopt(about = "Capture events live")]
pub enum CaptureArgs {
    /// Capture open window information from a (linux) X11 server
    X11(X11CaptureArgs),
    Windows(WindowsCaptureArgs),
}

#[enum_dispatch(CaptureArgs)]
pub trait CapturerCreator {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>>;
}

pub trait Capturer {
    fn capture(&mut self) -> anyhow::Result<EventData>;
}
