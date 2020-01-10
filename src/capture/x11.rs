// https://bitbucket.org/nomeata/arbtt/src/master/src/Capture/X11.hs
// https://docs.rs/x11rb/0.3.0/x11rb/
// Root Window Properties (and Related Messages) https://specifications.freedesktop.org/wm-spec/latest/ar01s03.html
use byteorder::{LittleEndian, ReadBytesExt};
use serde_json::{json, Value as J};
use std::collections::{BTreeMap, HashMap};
use std::convert::TryInto;
use x11rb::connection::Connection;
use x11rb::connection::RequestConnection;
use x11rb::errors::ConnectionErrorOrX11Error;
use x11rb::generated::xproto::get_property;
use x11rb::generated::xproto::intern_atom;
use x11rb::generated::xproto::ConnectionExt;
use x11rb::generated::xproto::ATOM;
use x11rb::generated::xproto::WINDOW;
use x11rb::xcb_ffi::XCBConnection;

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
fn split_zero(s: &str) -> Vec<String> {
    let mut vec: Vec<String> = s.split("\0").map(|e| String::from(e)).collect();
    let last = vec.pop().unwrap();
    if last.len() != 0 {
        panic!("not zero terminated");
    }
    return vec;
}

pub fn capture() -> anyhow::Result<()> {
    let (conn, screen_num) = XCBConnection::connect(None)?;
    let screen = &conn.setup().roots[screen_num];
    let root_window = screen.root;
    let atom = |e: &str| -> anyhow::Result<u32> {
        Ok(intern_atom(&conn, true, e.as_bytes())?.reply()?.atom)
    };

    let mut atom_name_map: HashMap<u32, anyhow::Result<String>> = HashMap::new();
    let mut atom_name = |e: u32| -> anyhow::Result<String> {
        let z = atom_name_map
            .entry(e.into())
            .or_insert_with(|| -> anyhow::Result<String> {
                Ok(String::from_utf8(conn.get_atom_name(e)?.reply()?.name)?)
            });
        match z {
            Err(_e) => Err(anyhow::anyhow!("idk dude")),
            Ok(ok) => Ok(ok.clone()),
        }
    };

    let NET_CLIENT_LIST = atom("_NET_CLIENT_LIST")?;
    let NET_WM_NAME = atom("_NET_WM_NAME")?;
    let WM_NAME = atom("WM_NAME")?;
    let WM_CLASS = atom("WM_CLASS")?;
    let NET_CURRENT_DESKTOP = atom("_NET_CURRENT_DESKTOP")?;
    let NET_DESKTOP_NAMES = atom("_NET_DESKTOP_NAMES")?;
    let NET_WM_WINDOW_TYPE = atom("_NET_WM_WINDOW_TYPE")?;
    let NET_WM_DESKTOP = atom("_NET_WM_DESKTOP")?;
    let current_desktop = single(&get_property32(&conn, root_window, NET_CURRENT_DESKTOP)?);
    let desktop_names = split_zero(&get_property_text(&conn, root_window, NET_DESKTOP_NAMES)?);
    //let mut WINDOW = x11rb::wrapper::LazyAtom::new(&conn, true, b"XCB_ATOM_WINDOW");
    let windows = get_property32(&conn, root_window, NET_CLIENT_LIST)?;

    println!(
        "current_desktop: {:?} ({:?})",
        current_desktop, desktop_names[current_desktop as usize]
    );
    println!("desktops: {:?}", desktop_names);
    println!("focus: {:x?}", conn.get_input_focus()?.reply()?.focus);
    let blacklist = vec![atom("_NET_WM_ICON")?];
    for window in windows {
        /*let name = get_property_text(&conn, window, NET_WM_NAME)?;
        let wtype = get_property32(&conn, window, NET_WM_WINDOW_TYPE)?;
        let wtypenames = wtype
            .into_iter()
            .map(|e| atom_name(e))
            .collect::<anyhow::Result<Vec<String>>>()?;
        let desktop = single(&get_property32(&conn, window, NET_WM_DESKTOP)?);
        let wmclass = split_zero(&get_property_text(&conn, window, WM_CLASS)?);
        let wmclass = format!("{}.{}", wmclass[0], wmclass[1]);
        println!("window: {:02x?} on {}: {}: {:?} {:?}", window, desktop_names[desktop as usize], wmclass, wtypenames, name);
        */
        let props = conn.list_properties(window)?.reply()?.atoms;
        let mut propmap: BTreeMap<String, J> = BTreeMap::new();
        for prop in props {
            if blacklist.contains(&prop) {
                continue;
            }
            let val = get_property(&conn, false, window, prop, 0, 0, std::u32::MAX)?.reply()?;
            assert!(val.bytes_after == 0);
            let prop_name = atom_name(prop)?;
            let prop_type = atom_name(val.type_)?;
            let pval = match prop_type.as_str() {
                "UTF8_STRING" | "STRING" => {
                    let s = String::from_utf8(val.value)?;
                    // if(s[s.len() - 1] == '\0') return 
                    J::String(s)
                },
                "ATOM" => {
                    assert!(val.format == 32);
                    let vec = to_u32s(&val.value);
                    //let vec = to_u32s(&val.value).into_iter().map(|e| J::Number(e)).collect();
                    json!({
                        "type": "ATOM",
                        "value": vec.into_iter().map(|e| atom_name(e)).collect::<anyhow::Result<Vec<_>>>()?
                    })
                }
                _ => {
                    println!("unknown type {}", prop_type);
                    json!({
                        "type": prop_type,
                        "value": hex::encode(val.value)
                    })
                }
            };
            propmap.insert(prop_name, pval);
        }
        println!(
            "propmap: {}",
            serde_json::to_string_pretty(&json!({"id": window, "properties": &propmap}))?
        );
    }

    Ok(())
}
