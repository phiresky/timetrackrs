// https://bitbucket.org/nomeata/arbtt/src/master/src/Capture/X11.hs
// https://docs.rs/x11rb/0.3.0/x11rb/
// Root Window Properties (and Related Messages) https://specifications.freedesktop.org/wm-spec/latest/ar01s03.html

#![allow(non_snake_case)]

use super::CapturedData;

use byteorder::{LittleEndian, ReadBytesExt};
use chrono::DateTime;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as J};
use std::collections::{BTreeMap, HashMap};
use std::convert::TryInto;
use sysinfo::ProcessExt;
use sysinfo::SystemExt;
use typescript_definitions::TypeScriptify;
use x11rb::connection::Connection;
use x11rb::connection::RequestConnection;
use x11rb::errors::ConnectionErrorOrX11Error;
use x11rb::generated::xproto::get_property;
use x11rb::generated::xproto::intern_atom;
use x11rb::generated::xproto::ConnectionExt;
use x11rb::generated::xproto::ATOM;
use x11rb::generated::xproto::WINDOW;
use x11rb::xcb_ffi::XCBConnection;

#[derive(Debug, Serialize, Deserialize, TypeScriptify)]
pub struct X11CapturedData {
    #[serde(default)]
    pub os_info: OsInfo,
    pub desktop_names: Vec<String>,
    pub current_desktop_id: usize,
    pub focused_window: u32,
    pub ms_since_user_input: u32,
    pub ms_until_screensaver: u32,
    pub screensaver_window: u32,
    pub windows: Vec<X11WindowData>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TypeScriptify)]
pub struct OsInfo {
    pub os_type: String,
    pub version: String,
    #[serde(default)]
    pub batteries: i32,
    pub hostname: String,
}
impl Default for OsInfo {
    fn default() -> OsInfo {
        OsInfo {
            os_type: "Arch Linux".to_string(),
            version: "1".to_string(),
            batteries: 0,
            hostname: "phirearch".to_string(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize, TypeScriptify)]
pub struct X11WindowData {
    pub window_id: u32,
    pub geometry: X11WindowGeometry,
    pub process: Option<ProcessData>,
    pub window_properties: BTreeMap<String, J>,
}
#[derive(Debug, Serialize, Deserialize, TypeScriptify)]
pub struct X11WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
#[derive(Debug, Serialize, Deserialize, TypeScriptify)]
pub struct ProcessData {
    pub pid: i32,
    pub name: String,
    pub cmd: Vec<String>,
    pub exe: String,
    pub cwd: String,
    pub memory_kB: i64,
    pub parent: Option<i32>,
    pub status: String,
    pub start_time: DateTime<chrono::Local>,
    pub cpu_usage: f32,
}
fn timestamp_to_iso_string(timestamp: u64) -> String {
    timestamp_to_iso(timestamp).to_rfc3339()
}
fn timestamp_to_iso(timestamp: u64) -> DateTime<chrono::Local> {
    use chrono::{DateTime, Local};
    use std::time::{Duration, UNIX_EPOCH};
    DateTime::<Local>::from(UNIX_EPOCH + Duration::from_secs(timestamp))
}
// TODO: replace with https://github.com/psychon/x11rb/issues/163
fn to_u32s(v1: &Vec<u8>) -> Vec<u32> {
    let mut c = std::io::Cursor::new(v1);
    let mut v2 = Vec::new();
    while c.position() < v1.len().try_into().unwrap() {
        v2.push(c.read_u32::<LittleEndian>().unwrap());
    }
    return v2;
}
fn get_property32<Conn: ?Sized + RequestConnection>(
    conn: &Conn,
    window: WINDOW,
    property: ATOM,
) -> Result<Vec<u32>, ConnectionErrorOrX11Error> {
    // TODO: use helper from https://github.com/psychon/x11rb/pull/172/files
    let reply = get_property(conn, false, window, property, 0, 0, std::u32::MAX)?.reply()?;

    Ok(to_u32s(&reply.value))
}
fn get_property_text<Conn: ?Sized + RequestConnection>(
    conn: &Conn,
    window: WINDOW,
    property: ATOM,
) -> Result<String, ConnectionErrorOrX11Error> {
    let reply = get_property(conn, false, window, property, 0, 0, std::u32::MAX)?.reply()?;

    Ok(String::from_utf8(reply.value).unwrap())
}
fn single<T: Copy>(v: &Vec<T>) -> T {
    if v.len() != 1 {
        panic!("not one response!!");
    }
    v[0]
}
// "2\u{0}4\u{0}5\u{0}6\u{0}8\u{0}9\u{0}1\u{0}" to array of strings
pub fn split_zero(s: &str) -> Vec<String> {
    let mut vec: Vec<String> = s.split("\0").map(|e| String::from(e)).collect();
    let last = vec.pop().unwrap();
    if last.len() != 0 {
        panic!("not zero terminated");
    }
    return vec;
}

pub struct X11Capturer {
    conn: XCBConnection,
    root_window: u32,
    atom_name_map: HashMap<u32, anyhow::Result<String>>,
    os_info: OsInfo,
}
impl X11Capturer {
    fn atom(&self, e: &str) -> anyhow::Result<u32> {
        Ok(intern_atom(&self.conn, true, e.as_bytes())?.reply()?.atom)
    }
    fn atom_name(&mut self, e: u32) -> anyhow::Result<String> {
        let conn = &self.conn;
        let z = self
            .atom_name_map
            .entry(e.into())
            .or_insert_with(|| -> anyhow::Result<String> {
                Ok(String::from_utf8(conn.get_atom_name(e)?.reply()?.name)?)
            });
        match z {
            Err(_e) => Err(anyhow::anyhow!("idk: {}", _e)),
            Ok(ok) => Ok(ok.clone()),
        }
    }
    pub fn init() -> anyhow::Result<X11Capturer> {
        let (conn, screen_num) = XCBConnection::connect(None)?;
        let screen = &conn.setup().roots[screen_num];
        let root_window = screen.root;
        let os_info1 = os_info::get();
        let batteries = battery::Manager::new()?.batteries()?.count();
        let os_info = OsInfo {
            os_type: os_info1.os_type().to_string(),
            version: format!("{}", os_info1.version()),
            hostname: hostname::get()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or("".to_string()),
            batteries: batteries as i32,
        };
        Ok(X11Capturer {
            conn,
            root_window,
            os_info,
            atom_name_map: HashMap::new(),
        })
    }
    pub fn capture(&mut self) -> anyhow::Result<CapturedData> {
        let system = sysinfo::System::new();
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
        windows.sort();

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
                let val =
                    get_property(&self.conn, false, window, prop, 0, 0, std::u32::MAX)?.reply()?;
                assert!(val.bytes_after == 0);
                let prop_name = self.atom_name(prop)?;
                let prop_type = self.atom_name(val.type_)?;
                if prop_name == "_NET_WM_PID" && prop_type == "CARDINAL" && val.format == 32 {
                    pid = Some(single(&to_u32s(&val.value)));
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
                        assert!(val.format == 32);
                        let vec = to_u32s(&val.value);
                        //let vec = to_u32s(&val.value).into_iter().map(|e| J::Number(e)).collect();
                        json!({
                            "type": format!("{}/{}", prop_type, val.format),
                            "value": vec.into_iter().map(|e| self.atom_name(e)).collect::<anyhow::Result<Vec<_>>>()?
                        })
                    }
                    (_, "CARDINAL", 32) | (_, "WINDOW", _) => {
                        assert!(val.format == 32);
                        let vec = to_u32s(&val.value);
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
                        start_time: timestamp_to_iso(procinfo.start_time()),
                        cpu_usage: procinfo.cpu_usage(),
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
            x11rb::generated::screensaver::query_info(&self.conn, self.root_window)?.reply()?;
        // see XScreenSaverQueryInfo at https://linux.die.net/man/3/xscreensaverunsetattributes

        let data = X11CapturedData {
            desktop_names: desktop_names,
            os_info: self.os_info.clone(),
            current_desktop_id: current_desktop as usize,
            focused_window: focus,
            ms_since_user_input: xscreensaver.ms_since_user_input,
            ms_until_screensaver: xscreensaver.ms_until_server,
            screensaver_window: xscreensaver.saver_window,
            windows: windowsdata,
        };
        Ok(CapturedData::x11(data))
    }
}

lazy_static::lazy_static! {
    static ref FORMATTED_TITLE_MATCH: Regex = Regex::new(r#"🛤([a-z]{2,5})🠚(.*)🠘"#).unwrap();
    static ref FORMATTED_TITLE_SPLIT: Regex = Regex::new("🙰").unwrap();
    static ref FORMATTED_TITLE_KV: Regex = Regex::new("^([a-z0-9]+)=(.*)$").unwrap();
    static ref SH_JSON_TITLE: Regex = Regex::new(r#"\{".*[^\\]"}"#).unwrap();
    static ref UNINTERESTING_BINARY: Regex = Regex::new(r#"/electron\d*$"#).unwrap();
    static ref BROWSER_BINARY: Regex = Regex::new(r#"/(firefox|google-chrome|chromium)$"#).unwrap();
    static ref URL: Regex =
        Regex::new(r#"(?i)https?://(-\.)?([^\s/?\.#-]+\.?)+(/[^\s]*)?"#).unwrap();
        // Regex::new(r#"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)"#).unwrap();
}
fn match_from_title(window: &X11WindowData, info: &mut ExtractedInfo) {
    use crate::extract::properties::*;
    if let Some(J::String(title)) = &window.window_properties.get("_NET_WM_NAME") {
        if let Some(sw) = &mut info.software {
            sw.title = title.clone();
            if let Some(p) = &window.process {
                sw.identifier = Identifier(p.exe.to_string());
                sw.unique_name = p.exe.to_string();
                if UNINTERESTING_BINARY.is_match(&p.exe) {
                    if let Some(J::String(cls)) = window.window_properties.get("WM_CLASS") {
                        let v = split_zero(cls);
                        sw.identifier = Identifier(format!("WM_CLASS:{}.{}", v[0], v[1]));
                    }
                }
            }
        };
        // match strictly formatted stuff in title:
        // 🛤sd🠚proj=/project/name🙰file=file/name🠘
        if let Some(cap) = FORMATTED_TITLE_MATCH.captures(title) {
            let category = cap.get(1).unwrap().as_str();
            let kv = {
                let mut kv = HashMap::new();
                let values = cap.get(2).unwrap().as_str();
                for kvs in FORMATTED_TITLE_SPLIT.split(values) {
                    let c = FORMATTED_TITLE_KV.captures(kvs).unwrap();
                    let k = c.get(1).unwrap().as_str().to_string();
                    let v = c.get(2).unwrap().as_str().to_string();
                    kv.insert(k, v);
                }
                kv
            };
            match category {
                "sd" => {
                    info.software_development = Some(SoftwareDevelopment {
                        project_path: kv.get("proj").or(kv.get("project")).map(|e| e.to_string()),
                        file_path: kv.get("file").unwrap().to_string(),
                    })
                }
                _ => {
                    println!("unknown category in title info: {}", category);
                }
            }
        }
        if let Some(m) = SH_JSON_TITLE.find(title) {
            if let Ok(J::Object(o)) = serde_json::from_str(m.as_str()) {
                if let (
                    Some(J::String(cwd)),
                    Some(J::Number(histdb)),
                    Some(J::String(usr)),
                    Some(J::String(cmd)),
                ) = (o.get("cwd"), o.get("histdb"), o.get("usr"), o.get("cmd"))
                {
                    info.shell = Some(Shell {
                        cwd: cwd.to_string(),
                        cmd: cmd.to_string(),
                        zsh_histdb_session_id: Identifier(histdb.to_string()),
                    })
                }
            }
        }
        if let Some(p) = &window.process {
            if BROWSER_BINARY.is_match(&p.exe) {
                if let Some(cap) = URL.find(&title) {
                    if let Ok(url) = url::Url::parse(cap.as_str()) {
                        info.web_browser = Some(WebBrowser {
                            url: cap.as_str().to_string(),
                            origin: url.origin().ascii_serialization(),
                            service: url.domain().unwrap().to_string(),
                        });
                    }
                }
            }
        }
    }
}

use crate::extract::{properties::ExtractedInfo, ExtractInfo};
impl ExtractInfo for X11CapturedData {
    fn extract_info(&self, event_id: String) -> Option<ExtractedInfo> {
        use crate::extract::properties::*;
        let x = &self;
        if x.ms_since_user_input > 120 * 1000 {
            return None;
        }
        let window = x.windows.iter().find(|e| e.window_id == x.focused_window);
        let mut info = ExtractedInfo {
            event_id,
            ..Default::default()
        }; /* Software {
               pub device: Text100,
               pub device_type: SoftwareDeviceType,
               pub device_os: Text10,
               pub title: Text10000,
               pub identifier: Identifier, // unique identifier for software package e.g. android:com.package.id or pc:hostname/binary
               pub unique_name: Text100, // name of software that should be globally unique and generally recognizable (e.g. "Firefox")
           }*/
        info.software = Some(Software {
            hostname: x.os_info.hostname.clone(),
            device_type: if x.os_info.batteries > 0 {
                SoftwareDeviceType::Laptop
            } else {
                SoftwareDeviceType::Desktop
            },
            device_os: x.os_info.os_type.to_string(),
            identifier: Identifier("".to_string()),
            title: "".to_string(),
            unique_name: "".to_string(),
        });
        match window {
            None => return None,
            Some(w) => match_from_title(w, &mut info),
        };
        Some(info)
    }
}
