use serde::Serialize;
use track_pc_usage_rs as trbtt;
use typescript_definitions::TypeScriptify;
use typescript_definitions::TypeScriptifyTrait;

use trbtt::capture::x11;
use trbtt::capture::CapturedData;

const fs: &'static [fn() -> std::borrow::Cow<'static, str>] = &[
    CapturedData::type_script_ify,
    x11::X11CapturedData::type_script_ify,
    x11::X11WindowData::type_script_ify,
    x11::X11WindowGeometry::type_script_ify,
    x11::ProcessData::type_script_ify,
];

// const all_types: Vec<
fn main() {
    if cfg!(any(debug_assertions, feature = "export-typescript")) {
        for f in fs {
            println!("{}", f());
        }
    } else {
        println!("NOT IN DEBUG MODE, will not work!")
    }
}
