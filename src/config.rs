use std::time::Duration;

use crate::{prelude::*, server::server::ServerConfig};

#[derive(Debug, Serialize, Deserialize)]
pub struct TimetrackrsConfig {
    pub capturers: Vec<CaptureConfig>,
    pub server: Option<ServerConfig>,
}

pub fn default_config() -> TimetrackrsConfig {
    TimetrackrsConfig {
        capturers: vec![CaptureConfig {
            args: CaptureArgs::NativeDefault(NativeDefaultArgs {}),
            interval: Duration::from_secs(30),
        }],
        server: Some(ServerConfig {
            listen: vec!["127.0.0.1:52714".to_string()],
        }),
    }
}
