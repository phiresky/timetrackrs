

use crate::{prelude::*, server::server::ServerConfig};

pub struct TimetrackrsConfig {
    pub capturers: Vec<CaptureConfig>,
    pub server: Option<ServerConfig>,
}

fn default_capture_args() -> CaptureArgs {
    #[cfg(target_os = "linux")]
    return CaptureArgs::X11(X11CaptureArgs {
        only_focused_window: false,
    });
    #[cfg(target_os = "windows")]
    return CaptureArgs::Windows(WindowsCaptureArgs {});
}

pub fn default_config() -> TimetrackrsConfig {
    TimetrackrsConfig {
        capturers: vec![], /*vec![CaptureConfig {
                               args: default_capture_args(),
                               interval: Duration::from_secs_f32(30.0),
                           }]*/
        server: Some(ServerConfig {
            listen: vec!["127.0.0.1:52714".to_string()],
        }),
    }
}
