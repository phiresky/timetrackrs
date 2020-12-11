use std::{borrow::Cow, collections::HashMap};

use crate::{
    expand::{expand_str_captures, expand_str_ez},
    prelude::*,
};

use regex::Regex;

use super::fetchers::{get_external_fetcher, get_simple_fetcher};

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
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
    lazy_static! {
        static ref RULES: Vec<TagRuleGroupV1> =
            serde_json::from_str(include_str!("../../data/rules/default.json"))
                .expect("could not parse internal rules");
    }
    validate_tag_rules(RULES.iter().flat_map(|d| &d.rules).map(|e| &e.rule));
    RULES
        .iter()
        .map(|data| TagRuleGroup {
            global_id: "<internal>".to_string(),
            data: TagRuleGroupData::V1 {
                data: (*data).clone(),
            },
        })
        .collect()
}

#[derive(Debug, Serialize, Deserialize, Clone, TypeScriptify)]
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
    HasTag {
        tag: String,
        new_tag: String,
    },
    ExactTagValue {
        tag: String,
        value: String,
        new_tag: String,
    },
    TagValuePrefix {
        tag: String,
        prefix: String,
        new_tag: String,
    },
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
fn single<'a>(tag: impl Into<Cow<'a, str>>) -> Tags {
    let mut tags = Tags::new();
    tags.insert(tag.into().into_owned());
    return tags;
}
impl TagRule {
    fn apply<'a>(
        &self,
        db: &DatyBasy,
        tags: &'a Tags,
    ) -> anyhow::Result<Option<(Tags, Vec<&'a str>)>> {
        match self {
            TagRule::HasTag { tag, new_tag } => {
                if let Some((full_tag, tag_value)) = tags
                    .iter()
                    .filter_map(|t| Some((t, t.strip_prefix(tag)?.strip_prefix(":")?)))
                    .next()
                {
                    Ok(Some((
                        single(expand_str_ez(new_tag, |r| match r {
                            "value" => tag_value,
                            _ => "",
                        })),
                        vec![full_tag],
                    )))
                } else {
                    Ok(None)
                }
            }
            TagRule::ExactTagValue {
                tag,
                value,
                new_tag,
            } => {
                let search = format!("{}:{}", tag, value);
                if let Some(i) = tags.iter().find(|i| *i == &search) {
                    Ok(Some((single(new_tag), vec![i])))
                } else {
                    Ok(None)
                }
            }
            TagRule::TagValuePrefix {
                tag,
                prefix,
                new_tag,
            } => {
                // example: tag=foo,prefix=bar
                // input tag: foo:barbaz
                // (full_tag=foo:bar, tag_value=barbaz, suffix=baz)
                if let Some((full_tag, tag_value, suffix)) = tags
                    .iter()
                    .filter_map(|t| Some((t, t.strip_prefix(tag)?.strip_prefix(":")?)))
                    .filter_map(|(full, value)| Some((full, value, value.strip_prefix(prefix)?)))
                    .next()
                {
                    Ok(Some((
                        single(expand_str_ez(new_tag, |r| match r {
                            "value" => tag_value,
                            "prefix" => prefix,
                            "suffix" => suffix,
                            _ => "",
                        })),
                        vec![full_tag],
                    )))
                } else {
                    Ok(None)
                }
            }
            TagRule::TagRegex { regexes, new_tag } => {
                let caps = match_multi_regex(&regexes, &tags);
                match caps {
                    None => Ok(None),
                    Some((caps, reason_tags)) => Ok(Some((
                        single(expand_str_captures(&caps, new_tag)),
                        reason_tags,
                    ))),
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
            TagRule::HasTag { .. } => Ok(()),
            TagRule::ExactTagValue { .. } => Ok(()),
            TagRule::TagValuePrefix { .. } => Ok(()),
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

pub fn apply_tag_rules(db: &mut DatyBasy, tags: &mut Tags) -> anyhow::Result<()> {
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
                .apply(
                    db,
                    &tags.iter().map(|(tag, _why)| tag.to_string()).collect(),
                )
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
