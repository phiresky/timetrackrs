use std::fmt::Debug;

use super::tags::Tags;
use crate::{expand::get_capture, prelude::*};
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

pub trait ExternalFetcher: Sync + Send {
    fn get_id(&self) -> &'static str;
    fn get_regexes(&self) -> &[Regex];
    fn get_cache_key(&self, found: &[regex::Captures], tags: &Tags) -> Option<String>;
    fn fetch_data(&self, cache_key: &str) -> anyhow::Result<String>;
    fn process_data(&self, tags: &Tags, cache_key: &str, data: &str) -> anyhow::Result<Tags>;
}

impl Debug for dyn ExternalFetcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fetcher({})", self.get_id())
    }
}

pub trait SimpleFetcher: Sync + Send {
    fn get_id(&self) -> &'static str;
    fn get_regexes(&self) -> &[Regex];
    fn process(&self, found: &[regex::Captures], tags: &Tags) -> anyhow::Result<Tags>;
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
    fn get_regexes(&self) -> &[Regex] {
        lazy_static! {
            static ref REGEXES: Vec<Regex> =
                vec![Regex::new(r#"^browse-url:(?P<url>.*)$"#).unwrap()];
        }

        &REGEXES
    }
    fn process(&self, found: &[regex::Captures], _tags: &Tags) -> anyhow::Result<Tags> {
        let url = get_capture(found, "url").context("Url match invalid?")?;
        let mut tags = Tags::new();

        let host = PUBLIC_SUFFIXES
            .parse_url(url)
            .map_err(|e| anyhow::anyhow!("{}", e))
            .with_context(|| format!("parsing url '{}'", url))?;

        if let publicsuffix::Host::Domain(domain) = host {
            tags.insert(format!("browse-full-domain:{}", domain.to_string()));
            if let Some(root) = domain.root() {
                tags.insert(format!("browse-main-domain:{}", root));
            }
            if !domain.has_known_suffix() {
                tags.insert(format!("error-unknown-domain:{}", domain));
            }
        };
        Ok(tags)
    }
}
