use std::collections::HashMap;

use crate::{expand::expand_str, prelude::*};

use regex::Regex;

use super::fetchers::{get_external_fetcher, get_simple_fetcher};

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
            SimpleRegexRule(
                r#"^browse-main-domain:telegram\.org$"#,
                "use-service:Telegram",
            ),
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
            SimpleRegexRule(
                r#"^browse-full-domain:mail.google.com$"#,
                "use-service:Gmail",
            ),
            SimpleRegexRule(r#"^browse-main-domain:youtube.com$"#, "use-service:YouTube"),
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
                    fetcher_id: "url-domain-matcher".to_string(),
                },
            },
            TagRuleWithMeta {
                enabled: true,
                rule: TagRule::ExternalFetcher {
                    fetcher_id: "youtube-meta-json".to_string(),
                },
            },
            TagRuleWithMeta {
                enabled: true,
                rule: TagRule::ExternalFetcher {
                    fetcher_id: "wikidata-domain-to-id-v1".to_string(),
                },
            },
            TagRuleWithMeta {
                enabled: true,
                rule: TagRule::ExternalFetcher {
                    fetcher_id: "wikidata-id-to-class".to_string(),
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
            SimpleRegexRule(
                r#"^software-type:media-player*$"#,
                "category:Entertainment/Video",
            ),
            SimpleRegexRule(r#"^software-type:shell*$"#, "category:Productivity/Shell"),
            /*SimpleRegexRule(
                r#"^use-service:Telegram*$"#,
                "category:Communication/Instant Messaging",
            ),*/
            SimpleRegexRule(
                r#"^wikidata-category:instant messaging client*$"#,
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
#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
#[serde(tag = "type")]
pub enum TagRule {
    TagRegex {
        #[serde(with = "serde_regex")]
        regexes: Vec<regex::Regex>,
        new_tag: String,
    },
    InternalFetcher {
        fetcher_id: String,
    },
    ExternalFetcher {
        fetcher_id: String,
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
// match all regexes against tags. returns None if one of the regexes did not match
fn match_multi_regex<'a>(
    regexes: &[Regex],
    tags: &'a Tags,
) -> Option<(Vec<regex::Captures<'a>>, Vec<&'a str>)> {
    let mut caps: Vec<regex::Captures> = Vec::new();
    let mut matched_tags: Vec<&str> = Vec::new();
    'nextregex: for regex in regexes {
        for tag in tags.iter() {
            let new = regex.captures(tag);
            if let Some(cap) = new {
                caps.push(cap);
                matched_tags.push(tag);
                continue 'nextregex;
            }
        }
        // no match for this regex, abort
        return None;
    }
    Some((caps, matched_tags))
}
impl TagRule {
    fn apply<'a>(
        &self,
        db: &DatyBasy,
        tags: &'a Tags,
    ) -> anyhow::Result<Option<(Tags, Vec<&'a str>)>> {
        match self {
            TagRule::TagRegex { regexes, new_tag } => {
                let mut new_tag_replaced: String = String::new();
                let caps = match_multi_regex(&regexes, &tags);
                match caps {
                    None => Ok(None),
                    Some((caps, reason_tags)) => {
                        expand_str(&caps, new_tag, &mut new_tag_replaced);
                        let mut new_tags = Tags::new();
                        new_tags.insert(new_tag_replaced);
                        Ok(Some((new_tags, reason_tags)))
                    }
                }
            }

            TagRule::ExternalFetcher { fetcher_id } => {
                let fetcher =
                    get_external_fetcher(&fetcher_id).context("could not find fetcher")?;
                let regexes = fetcher.get_regexes();
                let caps = match_multi_regex(&regexes, &tags);
                log::trace!("fetcher {} matched regexes to {:?}", fetcher.get_id(), caps);
                match caps {
                    None => Ok(None),
                    Some((caps, reason_tags)) => {
                        let id = fetcher.get_id();
                        if let Some(inner_cache_key) = fetcher.get_cache_key(&caps, tags) {
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
                            let new_tags = fetcher
                                .process_data(&tags, &inner_cache_key, &data)
                                .context("processing data")?;
                            Ok(Some((new_tags, reason_tags)))
                        } else {
                            Ok(None)
                        }
                    }
                }
            }
            TagRule::InternalFetcher { fetcher_id } => {
                let fetcher =
                    get_simple_fetcher(fetcher_id).context("could not find internal fetcher")?;
                let regexes = fetcher.get_regexes();
                let caps = match_multi_regex(&regexes, &tags);
                match caps {
                    None => Ok(None),
                    Some((caps, reason_tags)) => Ok(Some((
                        fetcher.process(&caps, &tags).context("processing data")?,
                        reason_tags,
                    ))),
                }
            }
        }
    }
    fn validate(&self) -> anyhow::Result<()> {
        match self {
            TagRule::TagRegex { regexes, .. } => {
                for regex in regexes {
                    validate_tag_regex(regex)?;
                }
                Ok(())
            }
            TagRule::ExternalFetcher { fetcher_id } => {
                let fetcher = get_external_fetcher(fetcher_id).context("could not find fetcher")?;
                for regex in fetcher.get_regexes() {
                    validate_tag_regex(regex)?;
                }
                Ok(())
            }
            TagRule::InternalFetcher { fetcher_id } => {
                let fetcher =
                    get_simple_fetcher(fetcher_id).context("could not find internal fetcher")?;
                for regex in fetcher.get_regexes() {
                    validate_tag_regex(regex)?;
                }
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
    let mut tags = intrinsic_tags;
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
            match rule
                .apply(db, tags)
                .with_context(|| format!("applying rule {:?}", rule))
            {
                Err(e) => log::warn!("{:?}", e),
                Ok(None) => {}
                Ok(Some((new_tags, _))) => tags.extend(new_tags),
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

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
#[serde(tag = "type")]
pub enum TagAddReason {
    IntrinsicTag {
        raw_data_type: String,
    },
    AddedByRule {
        matched_tags: Vec<String>,
        rule: TagRule,
    },
}

pub fn get_tags_with_reasons(
    db: &mut DatyBasy,
    intrinsic_tags: Tags,
) -> anyhow::Result<HashMap<String, TagAddReason>> {
    let mut tags: HashMap<String, TagAddReason> = intrinsic_tags
        .into_iter()
        .map(|tag| {
            (
                tag,
                TagAddReason::IntrinsicTag {
                    raw_data_type: "foobar".to_string(),
                },
            )
        })
        .collect();
    let mut last_length = tags.len();
    let mut settled = false;
    let mut iterations = 0;
    db.fetch_all_tag_rules_if_thoink()?;
    let rules = db.get_all_tag_rules()?;
    while !settled && iterations < 50 {
        for rule in rules {
            match rule
                .apply(db, &tags.iter().map(|(tag, _why)| tag.to_string()).collect())
                .with_context(|| format!("applying rule {:?}", rule))
            {
                Err(e) => log::warn!("{:?}", e),
                Ok(None) => {}
                Ok(Some((new_tags, matched_tags))) => tags.extend(new_tags.into_iter().map(
                    |new_tag| -> (String, TagAddReason) {
                        (
                            new_tag,
                            TagAddReason::AddedByRule {
                                rule: (*rule).clone(),
                                matched_tags: matched_tags
                                    .iter()
                                    .map(|e| e.to_string())
                                    .collect::<Vec<String>>(),
                            },
                        )
                    },
                )),
            }
        }
        settled = tags.len() == last_length;
        last_length = tags.len();
        iterations += 1;
    }
    if !settled {
        log::warn!("warning: tags did not settle");
    }
    Ok(tags)
}
