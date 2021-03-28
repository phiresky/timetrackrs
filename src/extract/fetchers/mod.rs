use std::{fmt::Debug, time::Duration};

use super::tags::Tags;
use crate::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
pub mod wikidata;
pub mod youtube;

pub use wikidata::*;
pub use youtube::YoutubeFetcher;

pub fn get_external_fetcher(id: &str) -> Option<&'static dyn ExternalFetcher> {
    lazy_static::lazy_static! {
        static ref EXT_FETCHERS: HashMap<&'static str, Box<dyn ExternalFetcher>> = vec![
            Box::new(youtube::YoutubeFetcher) as Box<dyn ExternalFetcher>,
            Box::new(WikidataIdFetcher) as Box<dyn ExternalFetcher>,
            Box::new(WikidataCategoryFetcher) as Box<dyn ExternalFetcher>,
        ].into_iter().map(|e| (e.get_id(), e)).collect();
    }
    EXT_FETCHERS.get(id).map(|e| e.as_ref())
}

pub fn get_simple_fetcher(id: &str) -> Option<&'static dyn SimpleFetcher> {
    lazy_static::lazy_static! {
        static ref SIMPLE_FETCHERS: HashMap<&'static str, Box<dyn SimpleFetcher>> = vec![
            Box::new(URLDomainMatcher) as Box<dyn SimpleFetcher>,
        ].into_iter().map(|e| (e.get_id(), e)).collect();
    }
    SIMPLE_FETCHERS.get(id).map(|e| e.as_ref())
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FetchResultJson {
    Ok { value: String },
    TemporaryFailure { reason: String, until: Timestamptz },
    PermanentFailure { reason: String },
}

pub enum FetchError {
    /// might succeed again if tried after the given duration
    TemporaryFailure(Box<dyn std::error::Error>, Duration),
    /// will not ever succeed, don't try again
    PermanentFailure(Box<dyn std::error::Error>),
}

impl From<Result<String, FetchError>> for FetchResultJson {
    fn from(r: Result<String, FetchError>) -> Self {
        match r {
            Ok(s) => FetchResultJson::Ok { value: s },
            Err(FetchError::TemporaryFailure(e, d)) => FetchResultJson::TemporaryFailure {
                reason: format!("{}", e),
                until: Timestamptz(
                    Utc::now() + chrono::Duration::from_std(d).expect("too large time"),
                ),
            },
            Err(FetchError::PermanentFailure(e)) => FetchResultJson::PermanentFailure {
                reason: format!("{}", e),
            },
        }
    }
}
#[async_trait]
pub trait ExternalFetcher: Sync + Send {
    fn get_id(&self) -> &'static str;
    fn get_regexes(&self) -> &[TagValueRegex];
    fn get_possible_output_tags(&self) -> &[&str];
    fn get_cache_key(&self, found: &[regex::Captures], tags: &Tags) -> Option<String>;
    async fn fetch_data(&self, cache_key: &str) -> Result<String, FetchError>;
    async fn process_data(
        &self,
        tags: &Tags,
        cache_key: &str,
        data: &str,
    ) -> anyhow::Result<Vec<TagValue>>;
}

impl Debug for dyn ExternalFetcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fetcher({})", self.get_id())
    }
}

pub trait SimpleFetcher: Sync + Send {
    fn get_id(&self) -> &'static str;
    fn get_regexes(&self) -> &[TagValueRegex];
    fn get_possible_output_tags(&self) -> &[&str];
    fn process(&self, found: &[regex::Captures], tags: &Tags) -> anyhow::Result<Vec<TagValue>>;
}

impl Debug for dyn SimpleFetcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fetcher({})", self.get_id())
    }
}

lazy_static! {
    // in theory the public suffix list should be kept up to date regularily
    // but eh
    pub static ref PUBLIC_SUFFIXES: publicsuffix::List =
        publicsuffix::List::from_str(include_str!("../../../data/public_suffix_list.dat"))
            .unwrap();
}

pub struct URLDomainMatcher;
impl SimpleFetcher for URLDomainMatcher {
    fn get_id(&self) -> &'static str {
        "url-domain-matcher"
    }
    fn get_possible_output_tags(&self) -> &[&str] {
        &[
            "browse-full-domain",
            "browse-main-domain",
            "error-unknown-domain",
        ]
    }
    fn get_regexes(&self) -> &[TagValueRegex] {
        lazy_static! {
            static ref REGEXES: Vec<TagValueRegex> = vec![TagValueRegex {
                tag: "browse-url".to_string(),
                regex: Regex::new(r#"^(?P<url>.*)$"#).unwrap()
            }];
        }

        &REGEXES
    }
    fn process(&self, found: &[regex::Captures], _tags: &Tags) -> anyhow::Result<Vec<TagValue>> {
        let url = get_capture(found, "url").context("Url match invalid?")?;
        let mut tags: Vec<TagValue> = Vec::new();

        let host = PUBLIC_SUFFIXES
            .parse_url(url)
            .map_err(|e| anyhow::anyhow!("{}", e))
            .with_context(|| format!("parsing url '{}'", url))?;

        if let publicsuffix::Host::Domain(domain) = host {
            tags.add("browse-full-domain", domain.to_string());
            if let Some(root) = domain.root() {
                tags.add("browse-main-domain", root);
            }
            if !domain.has_known_suffix() {
                tags.add("error-unknown-domain", domain.to_string());
            }
        };
        Ok(tags)
    }
}

pub fn temporary<T>(duration_s: u64) -> impl Fn(T) -> FetchError
where
    T: Into<Box<dyn std::error::Error>>,
{
    return move |e: T| FetchError::TemporaryFailure(e.into(), Duration::from_secs(duration_s));
}
