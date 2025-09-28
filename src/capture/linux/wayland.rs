use tokio::process::Command;

use super::wayland_types::WaylandCaptureArgs;
use crate::{capture::process::get_process_data, prelude::*, util::OsInfo};

use async_trait::async_trait;
use wayland_client::{
    event_created_child,
    globals::{registry_queue_init, GlobalListContents},
    protocol::{wl_registry, wl_seat::WlSeat},
    Connection, Dispatch, EventQueue, Proxy, QueueHandle,
};
use wayland_protocols::ext::idle_notify::v1::client::ext_idle_notification_v1::Event as IdleNotificationV1Event;
use wayland_protocols::ext::idle_notify::v1::client::ext_idle_notification_v1::ExtIdleNotificationV1;
use wayland_protocols::ext::idle_notify::v1::client::ext_idle_notifier_v1::ExtIdleNotifierV1;
use wayland_protocols_wlr::foreign_toplevel::v1::client::{
    zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1,
    zwlr_foreign_toplevel_manager_v1::{
        Event, ZwlrForeignToplevelManagerV1, EVT_FINISHED_OPCODE, EVT_TOPLEVEL_OPCODE,
    },
};
// refs:
// https://github.com/ActivityWatch/aw-watcher-window-wayland

// Right now there is no standard way to get the list of windows on wayland. There's a proposal called wlr_foreign_toplevel_management_v1
// that works for a few WMs but not the major ones (Gnome, KDE)
// so until that shit is fixed, I'll just use the sway API since that's what I care about
// swaymsg -t get_tree
// hyprctl clients -j

// update 2025: wlr_foreign_toplevel_management_v1 is implemented, but since it does not allow getting pids i'll keep using wm-specific methods

pub fn init_sway(_options: WaylandCaptureArgs) -> anyhow::Result<SwayCapturer> {
    Ok(SwayCapturer::new()?)
}
pub fn init_hyprland(_options: WaylandCaptureArgs) -> anyhow::Result<HyprlandCapturer> {
    Ok(HyprlandCapturer::new()?)
}
pub struct SwayCapturer {
    event_queue: EventQueue<WaylandListener>,
    listener: WaylandListener,
    os_info: OsInfo,
    system: sysinfo::System,
}

pub struct WaylandForeignTopLevelManagerCapturer {
    event_queue: EventQueue<WaylandListener>,
    listener: WaylandListener,
    os_info: OsInfo,
    system: sysinfo::System,
}

pub struct HyprlandCapturer {
    event_queue: EventQueue<WaylandListener>,
    listener: WaylandListener,
    os_info: OsInfo,
    system: sysinfo::System,
}

struct WaylandListener {
    inner_timeout: std::time::Duration,
    idle_notification: ExtIdleNotificationV1,
    last_input: Instant,
    is_idle: bool,
}
fn deep_collect_pids_sway(obj: &serde_json::Value) -> Vec<usize> {
    match obj {
        J::Array(e) => e.iter().flat_map(deep_collect_pids_sway).collect(),
        J::Object(e) => e
            .iter()
            .flat_map(|(k, v)| {
                if k == "pid" {
                    return v
                        .as_number()
                        .and_then(|e| e.as_u64())
                        .map(|e| vec![e as usize])
                        .unwrap_or_else(|| {
                            log::error!("could not parse pid {k}");
                            vec![]
                        });
                } else {
                    return deep_collect_pids_sway(v);
                }
            })
            .collect(),
        _ => vec![],
    }
}
fn deep_collect_pids_hyprland(obj: &serde_json::Value) -> Vec<usize> {
    obj.as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|e| {
            e.get("pid")?
                .as_number()
                .and_then(|e| e.as_u64())
                .map(|e| e as usize)
        })
        .collect()
}
impl SwayCapturer {
    fn new() -> anyhow::Result<SwayCapturer> {
        let connection = Connection::connect_to_env()
            .with_context(|| "Unable to connect to Wayland compositor")?;
        let (globals, mut event_queue) =
            registry_queue_init::<WaylandListener>(&connection).unwrap();
        let queue_handle = event_queue.handle();
        let seat: WlSeat = globals.bind(&queue_handle, 1..=WlSeat::interface().version, ())?;

        let idle: ExtIdleNotifierV1 = globals.bind(
            &queue_handle,
            1..=ExtIdleNotifierV1::interface().version,
            (),
        )?;
        /*let _windows: ZwlrForeignToplevelManagerV1 = globals.bind(
            &queue_handle,
            1..=ZwlrForeignToplevelManagerV1::interface().version,
            (),
        )?;*/

        let inner_timeout = std::time::Duration::from_secs(30);
        let idle_notification =
            idle.get_idle_notification(inner_timeout.as_millis() as u32, &seat, &queue_handle, ());
        let mut listener = WaylandListener {
            inner_timeout,
            idle_notification,
            last_input: Instant::now(),
            is_idle: false,
        };
        event_queue.roundtrip(&mut listener)?;
        let s = SwayCapturer {
            system: sysinfo::System::new(),
            event_queue,
            listener,
            os_info: util::get_os_info(),
        };
        Ok(s)
    }
}

impl HyprlandCapturer {
    fn new() -> anyhow::Result<HyprlandCapturer> {
        let connection = Connection::connect_to_env()
            .with_context(|| "Unable to connect to Wayland compositor")?;
        let (globals, mut event_queue) =
            registry_queue_init::<WaylandListener>(&connection).unwrap();
        let queue_handle = event_queue.handle();
        let seat: WlSeat = globals.bind(&queue_handle, 1..=WlSeat::interface().version, ())?;

        let idle: ExtIdleNotifierV1 = globals.bind(
            &queue_handle,
            1..=ExtIdleNotifierV1::interface().version,
            (),
        )?;
        /*let _windows: ZwlrForeignToplevelManagerV1 = globals.bind(
            &queue_handle,
            1..=ZwlrForeignToplevelManagerV1::interface().version,
            (),
        )?;*/

        let inner_timeout = std::time::Duration::from_secs(30);
        let idle_notification =
            idle.get_idle_notification(inner_timeout.as_millis() as u32, &seat, &queue_handle, ());
        let mut listener = WaylandListener {
            inner_timeout,
            idle_notification,
            last_input: Instant::now(),
            is_idle: false,
        };
        event_queue.roundtrip(&mut listener)?;
        let s = HyprlandCapturer {
            system: sysinfo::System::new(),
            event_queue,
            listener,
            os_info: util::get_os_info(),
        };
        Ok(s)
    }
}

impl WaylandForeignTopLevelManagerCapturer {
    fn new() -> anyhow::Result<WaylandForeignTopLevelManagerCapturer> {
        let connection = Connection::connect_to_env()
            .with_context(|| "Unable to connect to Wayland compositor")?;
        let (globals, mut event_queue) =
            registry_queue_init::<WaylandListener>(&connection).unwrap();
        let queue_handle = event_queue.handle();
        let seat: WlSeat = globals.bind(&queue_handle, 1..=WlSeat::interface().version, ())?;

        let idle: ExtIdleNotifierV1 = globals.bind(
            &queue_handle,
            1..=ExtIdleNotifierV1::interface().version,
            (),
        )?;
        let _windows: ZwlrForeignToplevelManagerV1 = globals.bind(
            &queue_handle,
            1..=ZwlrForeignToplevelManagerV1::interface().version,
            (),
        )?;

        let inner_timeout = std::time::Duration::from_secs(30);
        let idle_notification =
            idle.get_idle_notification(inner_timeout.as_millis() as u32, &seat, &queue_handle, ());
        let mut listener = WaylandListener {
            inner_timeout,
            idle_notification,
            last_input: Instant::now(),
            is_idle: false,
        };
        event_queue.roundtrip(&mut listener)?;
        panic!("not implemented fully");
        /*let s = SwayCapturer {
            system: sysinfo::System::new(),
            event_queue,
            listener,
            os_info: util::get_os_info(),
        };Ok(s)*/
    }
}

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
        // self.idle_watcher.run_iteration()?;
        // we don't really need to pump the event queue because we
        self.event_queue.roundtrip(&mut self.listener)?;
        let pids = deep_collect_pids_sway(&parsed);
        Ok(EventData::sway_v1(SwayEventData {
            window_tree: parsed,
            processes: pids
                .into_iter()
                .filter_map(|pid| get_process_data(&mut self.system, pid))
                .collect(),
            os_info: self.os_info.clone(),
            ms_since_user_input: if self.listener.is_idle {
                (Instant::now() - self.listener.last_input).as_millis() as u32
            } else {
                0
            },
            network: linux::network::get_network_info()
                .map_err(|e| log::info!("could not get net info: {}", e))
                .ok(),
        }))
    }
}

#[async_trait]
impl Capturer for HyprlandCapturer {
    async fn capture(&mut self) -> anyhow::Result<EventData> {
        let res = Command::new("hyprctl")
            .arg("clients")
            .arg("-j")
            .output()
            .await?;
        if !res.status.success() {
            anyhow::bail!(
                "Could not run hyprctl: {} {}",
                String::from_utf8_lossy(&res.stdout),
                String::from_utf8_lossy(&res.stderr)
            )
        }
        let parsed: serde_json::Value = serde_json::from_slice(&res.stdout)?;
        // self.idle_watcher.run_iteration()?;
        // we don't really need to pump the event queue because we
        self.event_queue.roundtrip(&mut self.listener)?;
        let pids = deep_collect_pids_hyprland(&parsed);
        Ok(EventData::hyprland_v1(HyprlandEventData {
            window_tree: parsed,
            processes: pids
                .into_iter()
                .filter_map(|pid| get_process_data(&mut self.system, pid))
                .collect(),
            os_info: self.os_info.clone(),
            ms_since_user_input: if self.listener.is_idle {
                (Instant::now() - self.listener.last_input).as_millis() as u32
            } else {
                0
            },
            network: linux::network::get_network_info()
                .map_err(|e| log::info!("could not get net info: {}", e))
                .ok(),
        }))
    }
}

impl Drop for WaylandListener {
    fn drop(&mut self) {
        log::info!("Releasing idle notification");
        self.idle_notification.destroy();
    }
}

impl WaylandListener {
    fn idle(&mut self) {
        self.is_idle = true;
        self.last_input = Instant::now() - self.inner_timeout;
        log::debug!("Got Wayland Idle Event");
    }

    fn resume(&mut self) {
        self.last_input = Instant::now();
        self.is_idle = false;
        log::debug!("Got Wayland Resumed Event");
    }
}

impl Dispatch<WlSeat, ()> for WaylandListener {
    fn event(
        _state: &mut Self,
        _proxy: &WlSeat,
        _event: <WlSeat as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}
impl Dispatch<ExtIdleNotifierV1, ()> for WaylandListener {
    fn event(
        _state: &mut Self,
        _proxy: &ExtIdleNotifierV1,
        _event: <ExtIdleNotifierV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ExtIdleNotificationV1, ()> for WaylandListener {
    fn event(
        state: &mut Self,
        _: &ExtIdleNotificationV1,
        event: <ExtIdleNotificationV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // println!("got idle notification event: {:?}", event);
        if let IdleNotificationV1Event::Idled = event {
            state.idle();
        } else if let IdleNotificationV1Event::Resumed = event {
            state.resume();
        }
    }
}
impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for WaylandListener {
    fn event(
        _state: &mut Self,
        _: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        l: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // When receiving events from the wl_registry, we are only interested in the
        // `global` event, which signals a new available global.
        // When receiving this event, we just print its characteristics in this example.
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            println!("[{}] {} (v{})", name, interface, version);
        }
        println!("globals: {:?}", l);
    }
}

impl Dispatch<ZwlrForeignToplevelManagerV1, ()> for WaylandListener {
    fn event(
        state: &mut Self,
        _x: &ZwlrForeignToplevelManagerV1,
        event: <ZwlrForeignToplevelManagerV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        println!("got foreign toplevel manager event: {:?}", event);
        match event {
            Event::Toplevel { toplevel } => {
                println!("New toplevel window: {:?}", toplevel);
                // toplevel.send_request(req)
            }
            Event::Finished => {
                log::info!("Foreign toplevel manager finished");
            }
            _ => {}
        }
    }
    event_created_child!(Self, ZwlrForeignToplevelManagerV1, [
        EVT_TOPLEVEL_OPCODE => (ZwlrForeignToplevelHandleV1, ()),
        EVT_FINISHED_OPCODE => (ZwlrForeignToplevelHandleV1, ())
    ]);
    /*fn event_created_child(
        opcode: u16,
        _qhandle: &QueueHandle<Self>,
    ) -> std::sync::Arc<dyn wayland_client::backend::ObjectData> {
        println!("debug: {:?}", opcode);
        _qhandle.make_data::<ZwlrForeignToplevelHandleV1, _>(())
    }*/
}

impl Dispatch<ZwlrForeignToplevelHandleV1, ()> for WaylandListener {
    fn event(
        state: &mut Self,
        _: &ZwlrForeignToplevelHandleV1,
        event: <ZwlrForeignToplevelHandleV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        println!("got foreign toplevel handle event: {:?}", event);
        match event {
            _ => {}
        }
    }
}
