use std::fmt::Display;

use crate::{db::fetcher_cache, prelude::*};

use diesel::SqliteConnection;
use regex::Regex;
lazy_static! {
    static ref public_suffixes: publicsuffix::List =
        publicsuffix::List::from_str(include_str!("../../data/public_suffix_list.dat")).unwrap();
}
fn get_tag_rules() -> Vec<TagRule> {
    return vec![
        TagRule::TagRegex {
            regex: Regex::new(r#"^browse-domain:telegram.org$"#).unwrap(),
            new_tag: "use-service:Telegram".to_string(),
        },
        TagRule::TagRegex {
            regex: Regex::new(r#"^use-executable:.*/telegram-desktop$"#).unwrap(),
            new_tag: "use-service:Telegram".to_string(),
        },
        TagRule::TagRegex {
            regex: Regex::new(r#"^browse-domain:mail.google.com$"#).unwrap(),
            new_tag: "use-service:Gmail".to_string(),
        },
        TagRule::TagRegex {
            regex: Regex::new(r#"^browse-domain:(.*\.)?youtube.com$"#).unwrap(),
            new_tag: "use-service:YouTube".to_string(),
        },
        TagRule::ExternalFetcher {
            regex: Regex::new(r#"^browse-domain:(.*\.)?youtube.com$"#).unwrap(),
            fetcher: Box::new(fetchers::YoutubeFetcher),
        },
        TagRule::TagRegex {
            regex: Regex::new(r#"^software-development-project:.*/(.*)$"#).unwrap(),
            new_tag: "software-development-project-name:$1".to_string(),
        },
    ];
}

#[derive(Debug)]
enum TagRule {
    TagRegex {
        regex: regex::Regex,
        new_tag: String,
    },
    ExternalFetcher {
        regex: regex::Regex,
        fetcher: Box<dyn fetchers::Fetcher>,
    },
}
pub type Tags = std::collections::BTreeSet<String>;

impl TagRule {
    fn apply(&self, db: &mut SqliteConnection, tags: &mut Tags) -> anyhow::Result<()> {
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
            TagRule::ExternalFetcher { regex, fetcher } => {
                for tag in tags.iter() {
                    if regex.is_match(tag) {
                        let id = fetcher.get_id();
                        if let Some(inner_cache_key) = fetcher.get_cache_key(tags) {
                            let global_cache_key = &format!("{}:{}", id, inner_cache_key);
                            log::trace!(
                                "matcher {} matched, cache key = {:?}",
                                id,
                                global_cache_key
                            );
                            let cached_data = fetcher_cache::get_cache_entry(db, global_cache_key)
                                .context("get cache entry")?;
                            let data = match cached_data {
                                Some(data) => data,
                                None => {
                                    let data = fetcher
                                        .fetch_data(&inner_cache_key)
                                        .context("fetching data")?;
                                    fetcher_cache::set_cache_entry(db, &global_cache_key, &data)
                                        .context("saving to cache")?;
                                    data
                                }
                            };
                            new_tags.extend(
                                fetcher
                                    .process_data(&tags, &inner_cache_key, &data)
                                    .context("processing data")?
                                    .into_iter(),
                            );
                            break;
                        }
                    }
                }
            }
        }
        tags.extend(new_tags);
        Ok(())
    }
}

pub fn get_tags(db: &mut SqliteConnection, e: &ExtractedInfo) -> Tags {
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
        general: GeneralSoftware { hostname, .. },
        specific:
            SpecificSoftware::SoftwareDevelopment {
                project_path: Some(path),
                ..
            },
        ..
    } = e
    {
        tags.insert(format!("software-development-project:{}{}", hostname, path));
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

    apply_tag_rules(db, &mut tags);
    tags
}

pub fn apply_tag_rules(db: &mut SqliteConnection, tags: &mut Tags) {
    let mut last_length = tags.len();
    let mut settled = false;
    let mut iterations = 0;
    let rules = get_tag_rules();
    while !settled && iterations < 50 {
        for rule in rules.iter() {
            if let Err(e) = rule
                .apply(db, tags)
                .with_context(|| format!("applying rule {:?}", rule))
            {}
        }
        settled = tags.len() == last_length;
        last_length = tags.len();
        iterations += 1;
    }
    if !settled {
        log::warn!("warning: tags did not settle");
    }
}
