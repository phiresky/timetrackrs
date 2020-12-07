use crate::{expand::expand_str, prelude::*};

use diesel::SqliteConnection;
use regex::Regex;

#[derive(Debug, Serialize, Deserialize, TypeScriptify)]
pub struct TagRuleGroupV1 {
    pub name: String,
    pub description: String,
    pub editable: bool,
    pub enabled: bool,
    pub rules: Vec<TagRuleWithMeta>,
}

fn validate_tag_rules<'a>(rules: impl IntoIterator<Item = &'a TagRule>) {
    for rule in rules {
        if let Err(e) = rule.validate() {
            log::warn!("tag rule {:?} is invalid: {}", rule, e);
        }
    }
}
pub fn get_default_tag_rule_groups() -> Vec<TagRuleGroup> {
    let rules = TagRuleGroupV1 {
        name: "Default Rules".to_string(),
        description: "These are shipped with the program :)".to_string(),
        editable: false,
        enabled: true,
        rules: vec![
            SimpleRegexRule(r#"^browse-domain:telegram\.org$"#, "use-service:Telegram"),
            SimpleRegexRule(
                r#"^software-executable-path:.*/(?P<basename>.*?)(?: \(deleted\)|.exe)?$"#,
                "software-executable-basename:$basename",
            ),
            SimpleRegexRule(
                r#"^software-executable-basename:telegram-desktop$"#,
                "use-service:Telegram",
            ),
            SimpleRegexRule(
                r#"^software-executable-basename:(firefox|google-chrome|chromium)$"#,
                "software-type:browser",
            ),
            SimpleRegexRule(
                r#"^software-executable-basename:(mpv|vlc)$"#,
                "software-type:media-player",
            ),
            SimpleRegexRule(
                r#"^software-window-title:.*(?P<url>https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)).*$"#,
                "browse-url:$url",
            ),
            SimpleRegexRule(r#"^browse-domain:mail.google.com$"#, "use-service:Gmail"),
            SimpleRegexRule(
                r#"^browse-domain:(.*\.)?youtube.com$"#,
                "use-service:YouTube",
            ),
            TagRuleWithMeta {
                enabled: true,
                rule: TagRule::TagRegex {
                    regexes: vec![
                        Regex::new(r#"^device-hostname:(?P<hostname>.*)$"#).unwrap(),
                        Regex::new(r#"^title-match-sd-proj:(?P<path>.*)$"#).unwrap(),
                    ],
                    new_tag: "software-development-project:$hostname$path".to_string(),
                },
            },
            TagRuleWithMeta {
                enabled: true,
                rule: TagRule::TagRegex {
                    regexes: vec![
                        Regex::new(r#"^software-executable-basename:electron\d*$"#).unwrap(),
                        Regex::new(r#"^software-window-class:(?P<class>.+)$"#).unwrap(),
                    ],
                    new_tag: "software-identifier:${class}".to_string(),
                },
            },
            TagRuleWithMeta {
                enabled: true,
                rule: TagRule::InternalFetcher {
                    regex: Regex::new(r#"^browse-url:(?P<url>.*)$"#).unwrap(),
                    fetcher: Box::new(fetchers::URLDomainMatcher),
                },
            },
            TagRuleWithMeta {
                enabled: true,
                rule: TagRule::ExternalFetcher {
                    regex: Regex::new(r#"^browse-domain:(.*\.)?youtube.com$"#).unwrap(),
                    fetcher: Box::new(fetchers::YoutubeFetcher),
                },
            },
            SimpleRegexRule(
                r#"^software-development-project:.*/(.*)$"#,
                "software-development-project-name:$1",
            ),
            SimpleRegexRule(r#"^title-match-shell-cwd:.*$"#, "software-type:shell"),
            SimpleRegexRule(r#"^software-development-project:.*$"#, "software-type:ide"),
            SimpleRegexRule(
                r#"^software-type:ide$"#,
                "category:Productivity/Software Development/IDE",
            ),
            SimpleRegexRule(r#"^software-type:shell*$"#, "category:Productivity/Shell"),
            SimpleRegexRule(r#"^software-type:media-player*$"#, "category:Entertainment"),
            SimpleRegexRule(r#"^software-type:shell*$"#, "category:Productivity/Shell"),
            SimpleRegexRule(
                r#"^use-service:Telegram*$"#,
                "category:Communication/Instant Messaging",
            ),
            SimpleRegexRule(r#"^use-service:Gmail*$"#, "category:Communication/Email"),
        ],
    };
    validate_tag_rules(rules.rules.iter().map(|e| &e.rule));
    vec![TagRuleGroup {
        global_id: "zdaarqppqxxayfbm".to_string(),
        data: TagRuleGroupData::V1 { data: rules },
    }]
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify)]
pub struct TagRuleWithMeta {
    //id: String,
    //name: Option<String>,
    //description: Option<String>,
    pub enabled: bool,
    pub rule: TagRule,
}
#[derive(Debug, Serialize, Deserialize, TypeScriptify)]
#[serde(tag = "type")]
pub enum TagRule {
    TagRegex {
        #[serde(with = "serde_regex")]
        regexes: Vec<regex::Regex>,
        new_tag: String,
    },
    InternalFetcher {
        #[serde(with = "serde_regex")]
        regex: regex::Regex,
        fetcher: Box<dyn fetchers::SimpleFetcher>,
    },
    ExternalFetcher {
        #[serde(with = "serde_regex")]
        regex: regex::Regex,
        fetcher: Box<dyn fetchers::ExternalFetcher>,
    },
}
pub type Tags = std::collections::BTreeSet<String>;

fn SimpleRegexRule(regex: &str, new_tag: &str) -> TagRuleWithMeta {
    TagRuleWithMeta {
        rule: TagRule::TagRegex {
            regexes: vec![Regex::new(regex).unwrap()],
            new_tag: new_tag.to_string(),
        },
        enabled: true,
    }
}
impl TagRule {
    fn apply(&self, db: &DatyBasy, tags: &mut Tags) -> anyhow::Result<()> {
        let mut new_tags = Vec::new();
        match self {
            TagRule::TagRegex { regexes, new_tag } => {
                let mut caps: Vec<regex::Captures> = Vec::new();
                'nextregex: for regex in regexes {
                    for tag in tags.iter() {
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
                            let cached_data = db
                                .get_cache_entry(global_cache_key)
                                .context("get cache entry")?;
                            let data = match cached_data {
                                Some(data) => data,
                                None => {
                                    let data = fetcher
                                        .fetch_data(&inner_cache_key)
                                        .context("fetching data")?;
                                    db.set_cache_entry(&global_cache_key, &data)
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
            TagRule::TagRegex { regexes, new_tag } => {
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
pub fn get_tags(db: &mut DatyBasy, intrinsic_tags: Tags) -> anyhow::Result<Tags> {
    let mut tags = Tags::new();
    tags.extend(intrinsic_tags);

    apply_tag_rules(db, &mut tags)?;
    Ok(tags)
}

pub fn apply_tag_rules<'a>(db: &mut DatyBasy, tags: &mut Tags) -> anyhow::Result<()> {
    let mut last_length = tags.len();
    let mut settled = false;
    let mut iterations = 0;
    db.fetch_all_tag_rules_if_thoink()?;
    let rules = db.get_all_tag_rules()?;
    while !settled && iterations < 50 {
        for rule in rules {
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
    Ok(())
}
