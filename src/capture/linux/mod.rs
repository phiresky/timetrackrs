#[cfg(target_os = "linux")]
pub mod network;
#[cfg(target_os = "linux")]
pub mod wayland;
#[cfg(target_os = "linux")]
pub mod x11;

// these types are cross platform
pub mod wayland_types;
pub mod x11_types;
