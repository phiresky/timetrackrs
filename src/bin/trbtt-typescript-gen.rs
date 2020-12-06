use std::io::prelude::*;
use track_pc_usage_rs as trbtt;
use trbtt::capture::*;
use trbtt::extract;
use trbtt::prelude::*;
use trbtt::sampler::Sampler;
use typescript_definitions::TypeScriptifyTrait;

#[cfg(debug_assertions)]
const FS: &'static [fn() -> std::borrow::Cow<'static, str>] = &[
    // DbEvent::type_script_ify,
    Sampler::type_script_ify,
    EventData::type_script_ify,
    x11_types::X11EventData::type_script_ify,
    x11_types::X11WindowData::type_script_ify,
    x11_types::X11WindowGeometry::type_script_ify,
    x11_types::ProcessData::type_script_ify,
    util::OsInfo::type_script_ify,
    TagRuleGroup::type_script_ify,
    TagRuleGroupData::type_script_ify,
    TagRuleWithMeta::type_script_ify,
    TagRule::type_script_ify,
    TagRuleGroupV1::type_script_ify,
];

#[cfg(not(debug_assertions))]
const FS: &'static [fn() -> std::borrow::Cow<'static, str>] = &[];

// const all_types: Vec<
fn main() -> anyhow::Result<()> {
    util::init_logging();

    let mut ofile = std::fs::File::create("frontend/src/server.d.ts")?;
    writeln!(ofile, "type DateTime<T> = string;")?;
    writeln!(ofile, "type Local = unknown;")?;
    writeln!(ofile, "type Timestamptz = string;")?;
    writeln!(ofile, "type Regex = string;")?;
    writeln!(ofile, "type ExternalFetcher = string;")?;
    writeln!(ofile, "type InternalFetcher = string;")?;
    if cfg!(any(debug_assertions, feature = "export-typescript")) {
        if FS.len() == 0 {
            println!("Not in debug mode??");
        }
        for f in FS {
            writeln!(ofile, "{}", f())?;
        }
        println!("output {} types", FS.len());
    } else {
        println!("NOT IN DEBUG MODE, will not work!")
    }
    Ok(())
}
