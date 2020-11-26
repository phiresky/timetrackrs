use crate::prelude::*;

use regex::Regex;

lazy_static::lazy_static! {
    static ref tag_rules: Vec<TagRule> = vec![
        TagRule::TagRegex {
            regex: Regex::new(r#"^browse-domain:telegram.org$"#).unwrap(),
            new_tag: "use-service:Telegram".to_string()
        }
    ];
    static ref public_suffixes: publicsuffix::List = publicsuffix::List::from_str(include_str!("../../data/public_suffix_list.dat")).unwrap();
}

enum TagRule {
    TagRegex {
        regex: regex::Regex,
        new_tag: String,
    },
}
type Tags = std::collections::BTreeSet<String>;

impl TagRule {
    fn apply(&self, tags: &mut Tags) {
        let mut new_tags = Vec::new();
        match self {
            TagRule::TagRegex { regex, new_tag } => {
                for tag in tags.iter() {
                    let new = regex.replace(tag, new_tag.as_str());
                    if &new != tag {
                        new_tags.push(new.to_string());
                    }
                }
            }
        }
        tags.extend(new_tags);
    }
}

pub fn get_tags(e: &ExtractedInfo) -> Tags {
    let mut tags = Tags::new();
    if let ExtractedInfo::InteractWithDevice {
        specific:
            SpecificSoftware::WebBrowser {
                url: Some(url),
                domain,
                ..
            },
        ..
    } = e
    {
        tags.insert(format!("browse-url:{}", url.to_string()));
        if let Some(domain) = domain {
            tags.insert(format!("browse-full-domain:{}", domain.to_string()));
            if let Ok(parsed) = public_suffixes.parse_domain(domain) {
                if let Some(root) = parsed.root() {
                    tags.insert(format!("browse-domain:{}", root));
                }
                if !parsed.has_known_suffix() {
                    tags.insert(format!("error-unknown-domain:{}", parsed));
                }
            }
        }
    }
    if let ExtractedInfo::InteractWithDevice {
        general:
            GeneralSoftware {
                opened_filepath: Some(path),
                hostname,
                ..
            },
        ..
    } = e
    {
        tags.insert(format!("open-file:{}{}", hostname, path));
    }
    if let ExtractedInfo::InteractWithDevice {
        general:
            GeneralSoftware {
                device_type,
                hostname,
                identifier,
                ..
            },
        ..
    } = e
    {
        tags.insert(format!("use-software:{}", identifier.0));
        tags.insert(format!("use-device-name:{}", hostname));
        tags.insert(format!("use-device-type:{:?}", device_type));
    }

    apply_tag_rules(&mut tags);
    tags
}

pub fn apply_tag_rules(tags: &mut Tags) {
    let mut last_length = tags.len();
    let mut settled = false;
    let mut iterations = 0;
    while !settled && iterations < 50 {
        for rule in tag_rules.iter() {
            rule.apply(tags);
        }
        settled = tags.len() == last_length;
        last_length = tags.len();
        iterations += 1;
    }
    if !settled {
        eprintln!("warning: tags did not settle");
    }
}
