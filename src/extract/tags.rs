use std::fmt::Display;

use crate::{db::fetcher_cache, expand::expand_str, prelude::*};

use diesel::SqliteConnection;
use regex::Regex;

fn validate_tag_rules(rules: &[TagRule]) {
    for rule in rules {
        if let Err(e) = rule.validate() {
            log::warn!("tag rule {:?} is invalid: {}", rule, e);
        }
    }
}
fn get_tag_rules() -> Vec<TagRule> {
    let rules =vec![
        TagRule::TagRegex {
            regex: Regex::new(r#"^browse-domain:telegram.org$"#).unwrap(),
            new_tag: "use-service:Telegram".to_string(),
        },
        TagRule::TagRegex {
            /**
            filename of the executable with two exceptions:
             - if software is updated etc and the executable is replaced with a newer version, the executable path will have (deleted) appended on linux - remove that suffix
             - on windows, remove the .exe suffix because who cares? 
            */
            regex: Regex::new(r#"^software-executable-path:.*/(?P<basename>.*?)(?: \(deleted\)|.exe)?$"#).unwrap(),
            new_tag: "software-executable-basename:$basename".to_string(),
        },
        TagRule::TagRegex {
            regex: Regex::new(r#"^software-executable-basename:telegram-desktop$"#).unwrap(),
            new_tag: "use-service:Telegram".to_string(),
        },
        TagRule::TagRegex {
            regex: Regex::new(r#"^software-executable-basename:(firefox|google-chrome|chromium)$"#).unwrap(),
            new_tag: "software-type:browser".to_string(),
        },
        TagRule::TagRegex {
            regex: Regex::new(r#"^software-executable-basename:(mpv|vlc)$"#).unwrap(),
            new_tag: "software-type:media-player".to_string()
        },
        TagRule::TagRegex {
            regex: Regex::new(
                r#"^software-window-title:.*(?P<url>https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)).*$"#,
            ).unwrap(),
            new_tag: "browse-url:$url".to_string(),
        },
        TagRule::TagRegex {
            regex: Regex::new(r#"^browse-domain:mail.google.com$"#).unwrap(),
            new_tag: "use-service:Gmail".to_string(),
        },
        TagRule::TagRegex {
            regex: Regex::new(r#"^browse-domain:(.*\.)?youtube.com$"#).unwrap(),
            new_tag: "use-service:YouTube".to_string(),
        },
        TagRule::MultiTagRegex {
            regexes: vec![
                Regex::new(r#"^device-hostname:(?P<hostname>.*)$"#).unwrap(),
                Regex::new(r#"^title-match-sd-proj:(?P<path>.*)$"#).unwrap(),
            ],
            new_tag: "software-development-project:$hostname$path".to_string(),
        },
        TagRule::MultiTagRegex {
            regexes: vec![
                Regex::new(r#"^software-executable-basename:electron\d*$"#).unwrap(),
                Regex::new(r#"^software-window-class:(?P<class>.+)$"#).unwrap(),
            ],
            new_tag: "software-identifier:${class}".to_string(),
        },
        TagRule::InternalFetcher {
            regex: Regex::new(r#"^browse-url:(?P<url>.*)$"#).unwrap(),
            fetcher: Box::new(fetchers::URLDomainMatcher)
        },
        TagRule::ExternalFetcher {
            regex: Regex::new(r#"^browse-domain:(.*\.)?youtube.com$"#).unwrap(),
            fetcher: Box::new(fetchers::YoutubeFetcher),
        },
        TagRule::TagRegex {
            regex: Regex::new(r#"^software-development-project:.*/(.*)$"#).unwrap(),
            new_tag: "software-development-project-name:$1".to_string(),
        },
        TagRule::TagRegex {
            regex: Regex::new(r#"^title-match-shell-cwd:.*$"#).unwrap(),
            new_tag: "software-type:shell".to_string(),
        },
        TagRule::TagRegex {
            regex: Regex::new(r#"^software-development-project:.**$"#).unwrap(),
            new_tag: "software-type:ide".to_string()
        },
        TagRule::TagRegex {
            regex: Regex::new(r#"^software-type:software-development*$"#).unwrap(),
            new_tag: "category:Productivity/Software Development/IDE".to_string()
        },
        TagRule::TagRegex {
            regex: Regex::new(r#"^software-type:shell*$"#).unwrap(),
            new_tag: "category:Productivity/Shell".to_string()
        },
        TagRule::TagRegex {
            regex: Regex::new(r#"^software-type:media-player*$"#).unwrap(),
            new_tag: "category:Entertainment".to_string()
        },
        TagRule::TagRegex {
            regex: Regex::new(r#"^software-type:shell*$"#).unwrap(),
            new_tag: "category:Productivity/Shell".to_string()
        },
        TagRule::TagRegex {
            regex: Regex::new(r#"^use-service:Telegram*$"#).unwrap(),
            new_tag: "category:Communication/Instant Messaging".to_string()
        },
        TagRule::TagRegex {
            regex: Regex::new(r#"^use-service:Gmail*$"#).unwrap(),
            new_tag: "category:Communication/Email".to_string()
        }
    ];
    validate_tag_rules(&rules);
    rules
}

#[derive(Debug)]
enum TagRule {
    TagRegex {
        regex: regex::Regex,
        new_tag: String,
    },
    MultiTagRegex {
        regexes: Vec<regex::Regex>,
        new_tag: String,
    },
    InternalFetcher {
        regex: regex::Regex,
        fetcher: Box<dyn fetchers::SimpleFetcher>,
    },
    ExternalFetcher {
        regex: regex::Regex,
        fetcher: Box<dyn fetchers::ExternalFetcher>,
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
            TagRule::MultiTagRegex { regexes, new_tag } => {
                let mut caps: Vec<regex::Captures> = Vec::new();
                'nextregex: for regex in regexes {
                    'thisregex: for tag in tags.iter() {
                        let new = regex.captures(tag);
                        if let Some(cap) = new {
                            caps.push(cap);
                            continue 'nextregex;
                        }
                    }
                    return Ok(());
                    // no match for this regex, abort
                }
                let mut new_tag_replaced: String = String::new();
                expand_str(&caps, new_tag, &mut new_tag_replaced);
                new_tags.push(new_tag_replaced);
            }

            TagRule::ExternalFetcher { regex, fetcher } => {
                for tag in tags.iter() {
                    if let Some(cap) = regex.captures(tag) {
                        let id = fetcher.get_id();
                        if let Some(inner_cache_key) = fetcher.get_cache_key(&cap, tags) {
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
            TagRule::InternalFetcher { regex, fetcher } => {
                for tag in tags.iter() {
                    if let Some(cap) = regex.captures(tag) {
                        new_tags.extend(
                            fetcher
                                .process(&cap, &tags)
                                .context("processing data")?
                                .into_iter(),
                        );
                    }
                }
            }
        }
        tags.extend(new_tags);
        Ok(())
    }
    fn validate(&self) -> anyhow::Result<()> {
        match self {
            TagRule::TagRegex { regex, new_tag } => {
                validate_tag_regex(regex)?;
                Ok(())
            }
            TagRule::MultiTagRegex { regexes, new_tag } => {
                for regex in regexes {
                    validate_tag_regex(regex)?;
                }
                Ok(())
            }
            TagRule::InternalFetcher { regex, fetcher } => {
                validate_tag_regex(regex)?;
                Ok(())
            }
            TagRule::ExternalFetcher { regex, fetcher } => {
                validate_tag_regex(regex)?;
                Ok(())
            }
        }
    }
}

fn validate_tag_regex(regex: &Regex) -> anyhow::Result<()> {
    let str = regex.as_str();
    if str.chars().next().context("regex empty")? != '^' {
        anyhow::bail!("regex must start with ^");
    }
    if str.chars().last().context("regex empty")? != '$' {
        anyhow::bail!("regexes must end with $");
    }
    Ok(())
}
pub fn get_tags(db: &mut SqliteConnection, intrinsic_tags: Tags) -> Tags {
    let mut tags = Tags::new();
    tags.extend(intrinsic_tags);

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
            {
                log::warn!("{:?}", e);
            }
        }
        settled = tags.len() == last_length;
        last_length = tags.len();
        iterations += 1;
    }
    if !settled {
        log::warn!("warning: tags did not settle");
    }
}
