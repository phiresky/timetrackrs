pub mod pc_common;
pub mod x11;
use crate::prelude::*;

#[enum_dispatch]
#[derive(StructOpt)]
#[structopt(about = "Capture events live")]
pub enum CaptureArgs {
    /// Capture open window information from a (linux) X11 server
    X11(X11CaptureArgs),
}

#[enum_dispatch(CaptureArgs)]
pub trait CapturerCreator {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>>;
}

pub trait Capturer {
    fn capture(&mut self) -> anyhow::Result<EventData>;
}
