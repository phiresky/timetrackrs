use crate::extract::properties::GeneralSoftware;
use crate::extract::properties::SpecificSoftware;
use regex::Regex;
use serde_json::Value as J;
use std::collections::HashMap;

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

pub fn match_from_title(
    sw: &mut GeneralSoftware,
    window_title: &str,
    window_class: &Option<(String, String)>, // "class" of the window which usually identifies the software
    executable_path: Option<&str>,
) -> SpecificSoftware {
    use crate::extract::properties::*;

    sw.title = window_title.to_string();
    if let Some(exe) = executable_path {
        sw.identifier = Identifier(exe.to_string());
        sw.unique_name = exe.to_string();
        if UNINTERESTING_BINARY.is_match(exe) {
            if let Some(cls) = window_class {
                sw.identifier = Identifier(format!("WM_CLASS:{}.{}", cls.0, cls.1));
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
        match category {
            "sd" => {
                return SpecificSoftware::SoftwareDevelopment {
                    project_path: kv.get("proj").or(kv.get("project")).map(|e| e.to_string()),
                    file_path: kv.get("file").unwrap().to_string(),
                };
            }
            _ => {
                println!("unknown category in title info: {}", category);
            }
        }
    }
    if let Some(m) = SH_JSON_TITLE.find(window_title) {
        if let Ok(J::Object(o)) = serde_json::from_str(m.as_str()) {
            if let (
                Some(J::String(cwd)),
                Some(J::Number(histdb)),
                Some(J::String(usr)),
                Some(J::String(cmd)),
            ) = (o.get("cwd"), o.get("histdb"), o.get("usr"), o.get("cmd"))
            {
                return SpecificSoftware::Shell {
                    cwd: cwd.to_string(),
                    cmd: cmd.to_string(),
                    zsh_histdb_session_id: Identifier(histdb.to_string()),
                };
            }
        }
    }
    if let Some(exe) = &executable_path {
        if BROWSER_BINARY.is_match(&exe) {
            if let Some(cap) = URL.find(&window_title) {
                if let Ok(url) = url::Url::parse(cap.as_str()) {
                    return SpecificSoftware::WebBrowser {
                        url: cap.as_str().to_string(),
                        origin: url.origin().ascii_serialization(),
                        service: url.domain().unwrap().to_string(),
                    };
                }
            }
        }
    }
    SpecificSoftware::Unknown
}
