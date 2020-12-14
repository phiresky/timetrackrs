use std::{fmt::Display, iter::FromIterator};

use crate::prelude::*;

// TODO: maybe use IndexMap<String, IndexSet<String>
#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct Tags {
    #[ts(ts_type = "{[key in string]?: string[]}")]
    map: multimap::MultiMap<String, String>,
}
#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct TagValue {
    tag: String,
    value: String,
}
impl TagValue {
    pub fn new(tag: impl Into<String>, value: impl Into<String>) -> TagValue {
        TagValue {
            tag: tag.into(),
            value: value.into(),
        }
    }
    pub fn map_value<R>(&self, mapper: impl Fn(&str) -> R) -> TagValue
    where
        R: Into<String>,
    {
        TagValue {
            tag: self.tag.clone(),
            value: mapper(&self.value).into(),
        }
    }
}
impl Display for TagValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.tag, self.value)
    }
}
impl Tags {
    pub fn new() -> Tags {
        Tags {
            map: multimap::MultiMap::new(),
        }
    }
    pub fn single(key: impl Into<String>, value: impl Into<String>) -> Tags {
        let mut tags = Tags::new();
        tags.add(key, value);
        tags
    }
    pub fn add(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.map.insert(key.into(), value.into());
    }
    pub fn has(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }
    pub fn has_all<'a>(&mut self, mut keys: impl Iterator<Item = &'a str>) -> bool {
        keys.all(|tag| self.has(tag))
    }
    pub fn get_one_value_of(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(|e| e.as_str())
    }
    pub fn get_all_values_of(&self, key: &str) -> &[String] {
        self.map.get_vec(key).map(|e| &e[..]).unwrap_or(&[])
    }
    pub fn has_value(&self, key: &str, value: &str) -> bool {
        self.map
            .get_vec(key)
            .and_then(|e| e.iter().find(|v| v.as_str() == value))
            .is_some()
    }
    pub fn extend(&mut self, e: Vec<TagValue>) {
        self.map.extend(e.into_iter().map(|v| (v.tag, v.value)))
    }
    pub fn iter(&self) -> multimap::Iter<String, String> {
        self.map.iter()
    }
    pub fn into_iter(
        self,
    ) -> std::collections::hash_map::IntoIter<std::string::String, Vec<std::string::String>> {
        self.map.into_iter()
    }
    pub fn total_value_count(&self) -> usize {
        self.iter().count()
    }
    pub fn tag_count(&self) -> usize {
        self.map.len()
    }
}

impl FromIterator<(String, String)> for Tags {
    fn from_iter<I: IntoIterator<Item = (String, String)>>(iter: I) -> Self {
        let mut c = Tags::new();
        for (k, v) in iter {
            c.add(k, v);
        }
        c
    }
}

pub trait AddTag {
    fn add(&mut self, key: impl Into<String>, value: impl Into<String>);
}

impl AddTag for Vec<TagValue> {
    fn add(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.push(TagValue::new(key, value));
    }
}
