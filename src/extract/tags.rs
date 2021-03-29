use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    iter::FromIterator,
};

use crate::prelude::*;

// TODO: maybe use IndexMap<String, IndexSet<String>
#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
pub struct Tags {
    #[ts(ts_type = "{[key in string]?: string[]}")]
    map: HashMap<String, HashSet<String>>,
}
#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone, Hash)]
pub struct TagValue {
    pub tag: String,
    pub value: String,
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
            map: HashMap::new(),
        }
    }
    pub fn single(key: impl Into<String>, value: impl Into<String>) -> Tags {
        let mut tags = Tags::new();
        tags.add(key, value);
        tags
    }
    pub fn add(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.map.entry(key.into()).or_default().insert(value.into());
    }
    pub fn has(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }
    pub fn has_all<'a>(&mut self, mut keys: impl Iterator<Item = &'a str>) -> bool {
        keys.all(|tag| self.has(tag))
    }
    pub fn get_one_value_of(&self, key: &str) -> Option<&str> {
        self.map
            .get(key)
            .and_then(|e| e.iter().next().map(|s| s.as_str()))
    }
    pub fn get_all_values_of<'a>(&'a self, key: &'a str) -> Box<dyn Iterator<Item = &str> + 'a> {
        self.map
            .get(key)
            .map(|e| Box::new(e.iter().map(|e| e.as_str())) as Box<dyn Iterator<Item = &'a str>>)
            .unwrap_or_else(|| Box::new(std::iter::empty()))
    }
    pub fn has_value(&self, key: &str, value: &str) -> bool {
        self.map
            .get(key)
            .map(|e| e.contains(value))
            .unwrap_or(false)
    }
    pub fn extend(&mut self, e: Vec<TagValue>) {
        for tag in e {
            self.add(tag.tag, tag.value);
        }
    }
    pub fn iter(
        &self,
    ) -> std::collections::hash_map::Iter<std::string::String, HashSet<std::string::String>> {
        self.map.iter()
    }
    pub fn iter_values(&self) -> impl Iterator<Item = (&str, &str)> {
        self.map.iter().flat_map(|(tag, values)| {
            values
                .iter()
                .map(move |value| (tag.as_str(), value.as_str()))
        })
    }

    pub fn total_value_count(&self) -> usize {
        self.iter().count()
    }
    pub fn tag_count(&self) -> usize {
        self.map.len()
    }
}

impl Default for Tags {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for Tags {
    type Item = (String, HashSet<String>);
    type IntoIter =
        std::collections::hash_map::IntoIter<std::string::String, HashSet<std::string::String>>;
    fn into_iter(
        self,
    ) -> std::collections::hash_map::IntoIter<std::string::String, HashSet<std::string::String>>
    {
        self.map.into_iter()
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
