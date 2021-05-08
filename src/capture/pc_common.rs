#![allow(clippy::trivial_regex)]

use crate::prelude::*;
use regex::Regex;
use serde_json::Value as J;
use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref FORMATTED_TITLE_MATCH: Regex = Regex::new(r#"🛤([a-z]{2,5})🠚(.*)🠘"#).unwrap();

    static ref FORMATTED_TITLE_SPLIT: Regex = Regex::new("🙰").unwrap();
    static ref FORMATTED_TITLE_KV: Regex = Regex::new("^([a-z0-9]+)=(.*)$").unwrap();
    static ref JSON_TITLE: Regex = Regex::new(r#"\{".*[^\\]"}"#).unwrap();
}

fn match_cmdline_to_filepath(cwd: &str, cmdline: &[String]) -> anyhow::Result<String> {
    if cmdline.len() == 2 {
        // TODO: windows??
        // on windows all paths should be converted to sane unix paths (e.g. C:\foo -> /c:/foo)
        if cmdline[1].starts_with('/') {
            return Ok(cmdline[1].clone());
        }
        if !cmdline[1].starts_with('-') {
            // path joining shouldn't be os-specific
            return Ok(std::path::PathBuf::from(cwd)
                .join(&cmdline[1])
                .to_string_lossy()
                .to_string());
        }
        anyhow::bail!("only cmd arg '{}' starts with -", cmdline[1]);
    } else {
        anyhow::bail!("found {} cmd args not 1", cmdline.len())
    }
}

/**
 * todo: smarter logic based on open program category?
 */
pub fn is_idle(duration: Duration) -> bool {
    return duration > Duration::from_secs(120);
}

/**
try to get structured info about a program from title etc
*/
pub fn match_software(
    window_title: &str,
    window_class: &Option<(String, String)>, // "class" of the window which usually identifies the software
    executable_path: Option<&str>,
    cwd: Option<&str>,
    cmdline: Option<&[String]>,
) -> Vec<TagValue> {
    use crate::extract::tags::*;

    let mut tags = Vec::new();

    tags.add("software-window-title", window_title);

    if let Some(exe) = executable_path {
        tags.add("software-executable-path", exe);
    }
    if let Some(cls) = window_class {
        tags.add("software-window-class", format!("{}.{}", cls.0, cls.1));
    }
    if let Some(cwd) = cwd {
        if let Some(cmdline) = cmdline {
            if let Ok(path) = match_cmdline_to_filepath(cwd, cmdline) {
                tags.add("software-opened-file", path);
            }
        }
    }

    // match strictly formatted data in title:
    // 🛤sd🠚proj=/project/name🙰file=file/name🠘
    if let Some(cap) = FORMATTED_TITLE_MATCH.captures(window_title) {
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
        for (k, v) in &kv {
            tags.add(format!("title-match-{}-{}", category, k), v);
        }
    }
    if let Some(m) = JSON_TITLE.find(window_title) {
        if let Ok(J::Object(mut o)) = serde_json::from_str(m.as_str()) {
            let mut category = o.remove("t").or_else(|| o.remove("type"));
            if category.is_none() && o.contains_key("histdb") {
                // hack for legacy entries in phiresky db
                category = Some(J::String("shell".to_string()));
            }
            if let Some(J::String(category)) = category {
                for (k, v) in &o {
                    let txtv = match v {
                        // no "" around string
                        J::String(s) => s.to_string(),
                        any => format!("{}", any),
                    };
                    tags.add(format!("title-match-{}-{}", category, k), txtv);
                }
            }
        }
    }
    tags
}
