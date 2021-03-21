use crate::prelude::*;
use std::collections::HashMap;

use regex::Regex;

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
        static ref RULES: Vec<TagRuleGroup> =
            serde_json::from_str(include_str!("../../data/rules/default.json"))
                .expect("could not parse internal rules");
    }
    let iter = RULES.iter().flat_map(|r| match &r.data.0 {
        TagRuleGroupData::V1 { data } => data.rules.iter().map(|r| &r.rule),
    });
    validate_tag_rules(iter);
    RULES.clone()
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
pub struct TagValueRegex {
    pub tag: String,
    #[serde(with = "serde_regex")]
    pub regex: Regex,
}
#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
#[serde(tag = "type")]
pub enum TagRule {
    HasTag {
        tag: String,
        /// these are expanded with $value => one value of the tag
        new_tags: Vec<TagValue>,
    },
    ExactTagValue {
        tag: String,
        value: String,
        new_tags: Vec<TagValue>,
    },
    TagValuePrefix {
        tag: String,
        prefix: String,
        /// these are expanded with
        /// $prefix => the given prefix
        /// $suffix => the rest of the tag value
        /// $value => the full tag value ($prefix$suffix)
        new_tags: Vec<TagValue>,
    },
    TagRegex {
        regexes: Vec<TagValueRegex>,
        /// these are expanded with the named (?P<name>.*) matches from all regexes
        new_tags: Vec<TagValue>,
    },
    InternalFetcher {
        fetcher_id: String,
    },
    ExternalFetcher {
        fetcher_id: String,
    },
}

// match all regexes against tags. returns None if one of the regexes did not match
fn match_multi_regex<'a>(
    regexes: &'a [TagValueRegex],
    tags: &'a Tags,
) -> Option<(Vec<regex::Captures<'a>>, Vec<TagValue>)> {
    let mut caps: Vec<regex::Captures> = Vec::new();
    let mut matched_tags: Vec<TagValue> = Vec::new();
    'nextregex: for TagValueRegex { tag, regex } in regexes {
        for value in tags.get_all_values_of(tag) {
            let new = regex.captures(value);
            if let Some(cap) = new {
                caps.push(cap);
                matched_tags.add(tag, value);
                continue 'nextregex;
            }
        }
        // no match for this regex, abort
        return None;
    }
    Some((caps, matched_tags))
}

impl TagRule {
    /// returns a vec of new values as well as a vec of values that are the reason for the addition
    /// todo: the reason vector should borrow from the orig tags instead of copying
    async fn apply<'a>(
        &self,
        db: &DatyBasy,
        orig_tags: &'a Tags,
    ) -> anyhow::Result<Option<(Vec<TagValue>, Vec<TagValue>)>> {
        match self {
            TagRule::HasTag { tag, new_tags } => {
                if let Some(tag_value) = orig_tags.get_one_value_of(tag) {
                    let expanded_tags = new_tags.iter().map(|new_tag| {
                        new_tag.map_value(|value| {
                            expand_str_ez(value, |r| match r {
                                "value" => tag_value,
                                _ => "",
                            })
                        })
                    });
                    let reason = vec![TagValue::new(tag, tag_value)];
                    Ok(Some((expanded_tags.collect(), reason)))
                } else {
                    Ok(None)
                }
            }
            TagRule::ExactTagValue {
                tag,
                value,
                new_tags,
            } => {
                if orig_tags.has_value(tag, value) {
                    Ok(Some((new_tags.clone(), vec![TagValue::new(tag, value)])))
                } else {
                    Ok(None)
                }
            }
            TagRule::TagValuePrefix {
                tag,
                prefix,
                new_tags,
            } => {
                // example: tag=foo,prefix=bar
                // input tag: foo:barbaz
                // (full_tag=foo:bar, tag_value=barbaz, suffix=baz)
                if let Some((tag_value, suffix)) = orig_tags
                    .get_all_values_of(tag)
                    .filter_map(|value| Some((value, value.strip_prefix(prefix)?)))
                    .next()
                {
                    let expanded_tags = new_tags.iter().map(|new_tag| {
                        new_tag.map_value(|value| {
                            expand_str_ez(value, |r| match r {
                                "value" => tag_value,
                                "prefix" => prefix,
                                "suffix" => suffix,
                                _ => "",
                            })
                        })
                    });
                    let reason = vec![TagValue::new(tag, tag_value)];
                    Ok(Some((expanded_tags.collect(), reason)))
                } else {
                    Ok(None)
                }
            }
            TagRule::TagRegex { regexes, new_tags } => {
                let caps = match_multi_regex(&regexes, &orig_tags);
                match caps {
                    None => Ok(None),
                    Some((caps, reason_tags)) => Ok(Some((
                        new_tags
                            .iter()
                            .map(|new_tag| {
                                new_tag.map_value(|value| expand_str_captures(&caps, value))
                            })
                            .collect(),
                        reason_tags,
                    ))),
                }
            }

            TagRule::ExternalFetcher { fetcher_id } => {
                let fetcher =
                    get_external_fetcher(&fetcher_id).context("could not find fetcher")?;
                let regexes = fetcher.get_regexes();
                let caps = match_multi_regex(&regexes, &orig_tags);
                log::trace!("fetcher {} matched regexes to {:?}", fetcher.get_id(), caps);
                match caps {
                    None => Ok(None),
                    Some((caps, reason_tags)) => {
                        let id = fetcher.get_id();
                        if let Some(inner_cache_key) = fetcher.get_cache_key(&caps, orig_tags) {
                            let global_cache_key = &format!("{}:{}", id, inner_cache_key);
                            log::trace!(
                                "matcher {} matched, cache key = {:?}",
                                id,
                                global_cache_key
                            );
                            let cached_data = db
                                .get_cache_entry(global_cache_key)
                                .await
                                .context("get cache entry")?;
                            let data = match cached_data {
                                Some(data) => data,
                                None => {
                                    let data = fetcher
                                        .fetch_data(&inner_cache_key)
                                        .await
                                        .context("fetching data")?;
                                    db.set_cache_entry(&global_cache_key, &data)
                                        .await
                                        .context("saving to cache")?;
                                    data
                                }
                            };
                            let new_tags = fetcher
                                .process_data(&orig_tags, &inner_cache_key, &data)
                                .await
                                .context("processing data")?;
                            check_tags_match_filter(&new_tags, fetcher.get_possible_output_tags())?;
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
                let caps = match_multi_regex(&regexes, &orig_tags);
                match caps {
                    None => Ok(None),
                    Some((caps, reason_tags)) => {
                        let new_tags = fetcher
                            .process(&caps, &orig_tags)
                            .context("processing data")?;
                        check_tags_match_filter(&new_tags, fetcher.get_possible_output_tags())?;
                        Ok(Some((new_tags, reason_tags)))
                    }
                }
            }
        }
    }
    fn validate(&self) -> anyhow::Result<()> {
        match self {
            TagRule::TagRegex { regexes, .. } => {
                for regex in regexes {
                    validate_tag_regex(&regex.regex)?;
                }
                Ok(())
            }
            TagRule::ExternalFetcher { fetcher_id } => {
                let fetcher = get_external_fetcher(fetcher_id).context("could not find fetcher")?;
                for regex in fetcher.get_regexes() {
                    validate_tag_regex(&regex.regex)?;
                }
                Ok(())
            }
            TagRule::InternalFetcher { fetcher_id } => {
                let fetcher =
                    get_simple_fetcher(fetcher_id).context("could not find internal fetcher")?;
                for regex in fetcher.get_regexes() {
                    validate_tag_regex(&regex.regex)?;
                }
                Ok(())
            }
            TagRule::HasTag { .. } => Ok(()),
            TagRule::ExactTagValue { .. } => Ok(()),
            TagRule::TagValuePrefix { .. } => Ok(()),
        }
    }
}

fn check_tags_match_filter(
    new_tags: &[TagValue],
    get_possible_output_tags: &[&str],
) -> anyhow::Result<()> {
    for tag in new_tags {
        if !get_possible_output_tags.contains(&tag.tag.as_str()) {
            anyhow::bail!("output unexpected tag {}", tag);
        }
    }
    Ok(())
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
pub async fn get_tags(db: &DatyBasy, intrinsic_tags: Tags) -> (Tags, i32) {
    let mut tags = intrinsic_tags;
    let iterations = apply_tag_rules(db, &mut tags).await;
    (tags, iterations)
}

pub async fn apply_tag_rules(db: &DatyBasy, tags: &mut Tags) -> i32 {
    let mut last_length = tags.total_value_count();
    let mut settled = false;
    let mut iterations = 0;
    let rules = db.get_all_tag_rules().await;
    while !settled && iterations < 50 {
        for rule in rules.iter() {
            match rule
                .apply(db, tags)
                .await
                .with_context(|| format!("applying rule {:?}", rule))
            {
                Err(e) => log::warn!("{:?}", e),
                Ok(None) => {}
                Ok(Some((new_tags, _))) => tags.extend(new_tags),
            }
        }
        let new_length = tags.total_value_count();
        settled = new_length == last_length;
        last_length = new_length;
        iterations += 1;
    }
    if !settled {
        log::warn!("warning: tags did not settle");
    }
    iterations
}

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
#[serde(tag = "type")]
pub enum TagAddReason {
    IntrinsicTag {
        raw_data_type: String,
    },
    AddedByRule {
        matched_tags: Vec<TagValue>,
        rule: TagRule,
    },
}

pub async fn get_tags_with_reasons(
    db: &DatyBasy,
    intrinsic_tags: Tags,
) -> (Tags, HashMap<String, TagAddReason>, i32) {
    let mut tags = intrinsic_tags;
    let (reasons, iterations) = apply_tag_rules_get_reasons(db, &mut tags).await;
    (tags, reasons, iterations)
}

pub async fn apply_tag_rules_get_reasons(
    db: &DatyBasy,
    tags: &mut Tags,
) -> (HashMap<String, TagAddReason>, i32) {
    let mut last_length = tags.total_value_count();
    let mut settled = false;
    let mut iterations = 0;
    let rules = db.get_all_tag_rules().await;
    let mut tag_reasons: HashMap<String, TagAddReason> = tags
        .iter_values()
        .map(|tag| {
            (
                format!("{}:{}", tag.0, tag.1),
                TagAddReason::IntrinsicTag {
                    raw_data_type: "foobar".to_string(),
                },
            )
        })
        .collect();

    while !settled && iterations < 50 {
        for rule in rules.iter() {
            match rule
                .apply(db, tags)
                .await
                .with_context(|| format!("applying rule {:?}", rule))
            {
                Err(e) => log::warn!("{:?}", e),
                Ok(None) => {}
                Ok(Some((new_tags, matched_tags))) => {
                    for tag in &new_tags {
                        let tag_w_v = format!("{}", tag);
                        tag_reasons.insert(
                            tag_w_v,
                            TagAddReason::AddedByRule {
                                rule: (*rule).clone(),
                                matched_tags: matched_tags.clone(),
                            },
                        );
                    }
                    tags.extend(new_tags);
                }
            }
        }
        let new_length = tags.total_value_count();
        settled = new_length == last_length;
        last_length = new_length;
        iterations += 1;
    }
    if !settled {
        log::warn!("warning: tags did not settle");
    }
    (tag_reasons, iterations)
}
