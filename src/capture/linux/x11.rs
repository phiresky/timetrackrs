// https://bitbucket.org/nomeata/arbtt/src/master/src/Capture/X11.hs
// https://docs.rs/x11rb/0.3.0/x11rb/
// Root Window Properties (and Related Messages) https://specifications.freedesktop.org/wm-spec/latest/ar01s03.html

#![allow(non_snake_case)]

use super::types::*;
use crate::prelude::*;

use serde_json::{json, Value as J};
use std::collections::{BTreeMap, HashMap};
use sysinfo::ProcessExt;
use sysinfo::SystemExt;
use x11rb::connection::Connection;
use x11rb::connection::RequestConnection;
use x11rb::protocol::xproto::get_property;
use x11rb::protocol::xproto::intern_atom;
use x11rb::protocol::xproto::Atom;
use x11rb::protocol::xproto::AtomEnum;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::protocol::xproto::Window;

fn get_property32<Conn: ?Sized + RequestConnection>(
    conn: &Conn,
    window: Window,
    property: Atom,
) -> anyhow::Result<Vec<u32>> {
    // TODO: use helper from https://github.com/psychon/x11rb/pull/172/files
    let reply = get_property(
        conn,
        false,
        window,
        property,
        AtomEnum::ANY,
        0,
        std::u32::MAX,
    )?
    .reply()?;
    Ok(reply.value32().unwrap().collect())
}
fn get_property_text<Conn: ?Sized + RequestConnection>(
    conn: &Conn,
    window: Window,
    property: Atom,
) -> anyhow::Result<String> {
    let reply = get_property(
        conn,
        false,
        window,
        property,
        AtomEnum::ANY,
        0,
        std::u32::MAX,
    )?
    .reply()?;

    Ok(String::from_utf8(reply.value).unwrap())
}
fn single<T: Copy>(v: &[T]) -> T {
    if v.len() != 1 {
        panic!("not one response!!");
    }
    v[0]
}

pub struct X11Capturer<C: Connection> {
    options: X11CaptureArgs,
    conn: C,
    root_window: u32,
    atom_name_map: HashMap<u32, anyhow::Result<String>>,
    os_info: util::OsInfo,
}
impl<C: Connection> X11Capturer<C> {
    fn atom(&self, e: &str) -> anyhow::Result<u32> {
        Ok(intern_atom(&self.conn, true, e.as_bytes())?.reply()?.atom)
    }
    fn atom_name(&mut self, e: u32) -> anyhow::Result<String> {
        let conn = &self.conn;
        let z = self
            .atom_name_map
            .entry(e)
            .or_insert_with(|| -> anyhow::Result<String> {
                Ok(String::from_utf8(conn.get_atom_name(e)?.reply()?.name)?)
            });
        match z {
            Err(_e) => Err(anyhow::anyhow!("idk: {}", _e)),
            Ok(ok) => Ok(ok.clone()),
        }
    }
}
pub fn init(options: X11CaptureArgs) -> anyhow::Result<X11Capturer<impl Connection>> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_num];
    let root_window = screen.root;
    Ok(X11Capturer {
        options,
        conn,
        root_window,
        os_info: util::get_os_info(),
        atom_name_map: HashMap::new(),
    })
}

impl<C: Connection> Capturer for X11Capturer<C> {
    fn capture(&mut self) -> anyhow::Result<EventData> {
        let mut system = sysinfo::System::new();
        let NET_CLIENT_LIST = self.atom("_NET_CLIENT_LIST")?;
        let NET_CURRENT_DESKTOP = self.atom("_NET_CURRENT_DESKTOP")?;
        let NET_DESKTOP_NAMES = self.atom("_NET_DESKTOP_NAMES")?;

        let blacklist = vec![
            self.atom("_NET_WM_ICON")?, // HUUGE
            self.atom("WM_ICON_NAME")?, // invalid unicode _NET_WM_ICON_NAME
            self.atom("WM_NAME")?,      // invalid unicode, use _NET_WM_NAME
        ];

        let current_desktop = single(&get_property32(
            &self.conn,
            self.root_window,
            NET_CURRENT_DESKTOP,
        )?);
        let desktop_names = split_zero(&get_property_text(
            &self.conn,
            self.root_window,
            NET_DESKTOP_NAMES,
        )?);
        let focus = self.conn.get_input_focus()?.reply()?.focus;
        let mut windows = get_property32(&self.conn, self.root_window, NET_CLIENT_LIST)?;
        windows.sort_unstable();
        if self.options.only_focused_window {
            windows.retain(|i| i == &focus);
        }

        let mut windowsdata = vec![];
        if !windows.contains(&focus) {
            println!("Focussed thing is not in window list!!");
        }
        for window in windows {
            let props = self.conn.list_properties(window)?.reply()?.atoms;
            let mut propmap: BTreeMap<String, J> = BTreeMap::new();
            let mut pid = None;
            for prop in props {
                if blacklist.contains(&prop) {
                    continue;
                }
                let val = get_property(
                    &self.conn,
                    false,
                    window,
                    prop,
                    AtomEnum::ANY,
                    0,
                    std::u32::MAX,
                )?
                .reply()?;
                assert!(val.bytes_after == 0);
                let prop_name = self.atom_name(prop)?;
                let prop_type = self.atom_name(val.type_)?;
                if prop_name == "_NET_WM_PID" && prop_type == "CARDINAL" {
                    pid = val
                        .value32()
                        .map(|e| e.collect::<Vec<_>>())
                        .map(|e| single(&e));
                }
                let pval = match (prop_name.as_str(), prop_type.as_str(), val.format) {
                    (_, "UTF8_STRING", _) | (_, "STRING", _) => {
                        let QQQ = val.value.clone();
                        let s = String::from_utf8(val.value).map_err(|e| {
                            println!("str {} was!! {:x?}", &prop_name, QQQ);
                            e
                        })?;
                        // if(s[s.len() - 1] == '\0') return
                        J::String(s)
                    }
                    (_, "ATOM", _) => {
                        let vec = val.value32().expect("atom value not 32 bit");
                        //let vec = to_u32s(&val.value).into_iter().map(|e| J::Number(e)).collect();
                        json!({
                            "type": format!("{}/{}", prop_type, val.format),
                            "value": vec.into_iter().map(|e| self.atom_name(e)).collect::<anyhow::Result<Vec<_>>>()?
                        })
                    }
                    (_, "CARDINAL", 32) | (_, "WINDOW", _) => {
                        let vec: Vec<_> = val.value32().expect("atom value not 32 bit").collect();
                        //let vec = to_u32s(&val.value).into_iter().map(|e| J::Number(e)).collect();
                        json!({
                            "type": format!("{}/{}", prop_type, val.format),
                            "value": vec
                        })
                    }
                    _ => json!({
                        "type": format!("{}/{}", prop_type, val.format),
                        "value": hex::encode(val.value)
                    }),
                };
                propmap.insert(prop_name, pval);
            }

            let process = if let Some(pid) = pid {
                system.refresh_process(pid as i32);
                if let Some(procinfo) = system.get_process(pid as i32) {
                    Some(ProcessData {
                        pid: procinfo.pid(),
                        name: procinfo.name().to_string(),
                        cmd: procinfo.cmd().to_vec(),
                        exe: procinfo.exe().to_string_lossy().to_string(), // tbh i don't care if your executables have filenames that are not unicode
                        cwd: procinfo.cwd().to_string_lossy().to_string(),
                        memory_kB: procinfo.memory() as i64,
                        parent: procinfo.parent(),
                        status: procinfo.status().to_string().to_string(),
                        start_time: util::unix_epoch_millis_to_date(
                            (procinfo.start_time() as i64) * 1000,
                        ),
                        cpu_usage: Some(procinfo.cpu_usage()),
                    })
                } else {
                    println!(
                        "could not get process by pid {} for window {} ({})",
                        pid,
                        window,
                        json!(&propmap)
                    );
                    None
                }
            } else {
                None
            };

            let geo = self.conn.get_geometry(window)?.reply()?;
            let coords = self
                .conn
                .translate_coordinates(window, self.root_window, 0, 0)?
                .reply()?;

            windowsdata.push(X11WindowData {
                window_id: window,
                geometry: X11WindowGeometry {
                    x: coords.dst_x as i32,
                    y: coords.dst_y as i32,
                    width: geo.width as i32,
                    height: geo.height as i32,
                },
                process,
                window_properties: propmap,
            });
        }
        let xscreensaver =
            x11rb::protocol::screensaver::query_info(&self.conn, self.root_window)?.reply()?;
        // see XScreenSaverQueryInfo at https://linux.die.net/man/3/xscreensaverunsetattributes

        let data = X11EventData {
            desktop_names,
            os_info: self.os_info.clone(),
            current_desktop_id: current_desktop as usize,
            focused_window: focus,
            ms_since_user_input: xscreensaver.ms_since_user_input,
            ms_until_screensaver: xscreensaver.ms_until_server,
            screensaver_window: xscreensaver.saver_window,
            windows: windowsdata,
            network: linux::network::get_network_info()
                .map_err(|e| log::info!("could not get net info: {}", e))
                .ok(),
        };
        Ok(EventData::x11_v2(data))
    }
}
