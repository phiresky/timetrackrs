use super::ExternalFetcher;
use crate::prelude::*;
use itertools::Itertools;
use regex::Regex;

// ugh. https://phabricator.wikimedia.org/T196450

/*static seer = r#"
SELECT distinct ?service ?website_url ?outer_category ?outer_categoryLabel WHERE \{
    ?service wdt:P856 ?website_url.
    optional {{
        ?service wdt:P31 ?inner_category.
        ?inner_category wdt:P279* ?outer_category.
        ?outer_category wdt:P279+ wd:Q1668024.
    }}
    VALUES ?website_url {{ {} }}
    SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en" }}
}
"#;*/

pub struct WikidataIdFetcher;

#[async_trait]
impl ExternalFetcher for WikidataIdFetcher {
    fn get_id(&self) -> &'static str {
        "wikidata-domain-to-id-v1"
    }

    fn get_regexes(&self) -> &[TagValueRegex] {
        lazy_static::lazy_static! {
            static ref REGEXES: Vec<TagValueRegex> =
                vec![TagValueRegex{tag: "browse-full-domain".to_string(), regex: Regex::new(r#"^(?P<domain>.*)$"#).unwrap()}];
        }
        &REGEXES
    }

    fn get_possible_output_tags(&self) -> &[&str] {
        &["wikidata-website-url", "wikidata-id", "wikidata-label"]
    }

    fn get_cache_key(
        &self,
        found: &[regex::Captures],
        _tags: &crate::prelude::Tags,
    ) -> Option<String> {
        get_capture(found, "domain").map(|d| d.to_string())
    }

    async fn fetch_data(&self, cache_key: &str) -> Result<String, FetchError> {
        log::debug!("fetching {} from wikidata", cache_key);
        let api = mediawiki::api::Api::new("https://www.wikidata.org/w/api.php")
            .await
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("wikidata api initialization")
            .map_err(temporary(60))?; // Will determine the SPARQL API URL via site info data

        //let main_domain = public_suffixes.from_domain(cache_key);
        /*let query_api_url = api
        .get_site_info_string("general", "wikibase-sparql")
        .map_err(|e| anyhow::anyhow!("{}", e))?;
        let query_api_url = "https://query.wikidata.org/sparql";
        println!("wd url: {}", query_api_url);*/

        // wikidata is inconsistent, so try a few combinations of domains
        // (note that matching using filter(contains(str(?website_url), ...))) would cause high load for wikidata so we dont do that)
        let exact_domain_urls: Vec<String> = vec![
            format!("http://{}", cache_key),
            format!("https://{}", cache_key),
            format!("http://{}/", cache_key),
            format!("https://{}/", cache_key),
        ];
        let main_domain_urls: Vec<String> = super::PUBLIC_SUFFIXES
            .parse_domain(cache_key)
            .map(|e| {
                e.root().map(|root| {
                    vec![
                        format!("http://{}", root),
                        format!("https://{}", root),
                        format!("http://{}/", root),
                        format!("https://{}/", root),
                        format!("http://www.{}", root),
                        format!("https://www.{}/", root),
                    ]
                })
            })
            .ok()
            .flatten()
            .unwrap_or_default();

        let urlinner: String = exact_domain_urls
            .iter()
            .chain(main_domain_urls.iter())
            .map(|e| format!("<{}>", e))
            .join(" ");
        let query = format!(
            r#"
            SELECT distinct ?service ?serviceLabel ?website_url WHERE {{
                # service has official website url x
                ?service wdt:P856 ?website_url.
                VALUES ?website_url {{ {} }}
                SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en" }}
            }}                    
            "#,
            urlinner
        );
        log::trace!("full sparql query: {}", query);

        let mutres = api
            .sparql_query(&query)
            .await
            .context("Could not run SPARQL query")
            .map_err(temporary(60))?;

        let (full_domain_matches, main_domain_matches): (
            Vec<&serde_json::Value>,
            Vec<&serde_json::Value>,
        ) = mutres["results"]["bindings"]
            .as_array()
            .context("unparseable response")
            .map_err(temporary(60 * 60))?
            .iter()
            .partition(|e| {
                e["website_url"]["value"]
                    .as_str()
                    .and_then(|url| exact_domain_urls.iter().find(|u| *u == url))
                    .is_some()
            });
        Ok(format!(
            "{}",
            serde_json::json!({"main_domain_matches": &main_domain_matches, "full_domain_matches": &full_domain_matches})
        ))
    }

    async fn process_data(
        &self,
        _tags: &Tags,
        _cache_key: &str,
        data: &str,
    ) -> anyhow::Result<Vec<TagValue>> {
        let parsed: serde_json::Value = serde_json::from_str(data)?;

        let mut tags = Vec::new();

        let matches = parsed["full_domain_matches"]
            .as_array()
            .filter(|e| !e.is_empty())
            .or_else(|| parsed["main_domain_matches"].as_array())
            .context("internal error?")?;

        for matching in matches {
            if let Some(e) = matching["website_url"]["value"].as_str() {
                tags.push(TagValue::new("wikidata-website-url", e))
            }
            if let Some(e) = matching["service"]["value"].as_str() {
                tags.add(
                    "wikidata-id",
                    e.strip_prefix("http://www.wikidata.org/entity/")
                        .unwrap_or("returned id weird?"),
                )
            }

            if let Some(e) = matching["serviceLabel"]["value"].as_str() {
                tags.add("wikidata-label", e)
            }
        }
        Ok(tags)
    }
}

pub struct WikidataCategoryFetcher;
#[async_trait]
impl ExternalFetcher for WikidataCategoryFetcher {
    fn get_id(&self) -> &'static str {
        "wikidata-id-to-class"
    }

    fn get_regexes(&self) -> &[TagValueRegex] {
        lazy_static::lazy_static! {
            static ref REGEXES: Vec<TagValueRegex> =
                vec![TagValueRegex{tag: "wikidata-id".to_string(), regex: Regex::new(r#"^wikidata-id:(?P<id>.*)$"#).unwrap()}];
        }
        &REGEXES
    }

    fn get_possible_output_tags(&self) -> &[&str] {
        &["wikidata-category"]
    }

    fn get_cache_key(
        &self,
        found: &[regex::Captures],
        _tags: &crate::prelude::Tags,
    ) -> Option<String> {
        get_capture(found, "id").map(|d| d.to_string())
    }

    async fn fetch_data(&self, cache_key: &str) -> Result<String, FetchError> {
        let api = mediawiki::api::Api::new("https://www.wikidata.org/w/api.php")
            .await
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("wikidata api initialization")
            .map_err(|e| FetchError::TemporaryFailure(e.into(), Duration::from_secs(60)))?; // Will determine the SPARQL API URL via site info data

        let query = format!(
            r#"
            SELECT ?category ?categoryLabel WHERE {{
                # X instance of Y
                wd:{} wdt:P31 ?category.
                SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en" }}
            }}
            "#,
            cache_key
        );
        log::trace!("full sparql query: {}", query);

        let res = api
            .sparql_query(&query)
            .await
            .context("Could not run SPARQL query")
            .map_err(|e| FetchError::TemporaryFailure(e.into(), Duration::from_secs(60)))?;

        Ok(format!("{}", res["results"]["bindings"]))
    }

    async fn process_data(
        &self,
        _tags: &Tags,
        _cache_key: &str,
        data: &str,
    ) -> anyhow::Result<Vec<TagValue>> {
        // in theory we should use the ids. but eh that's sooo ugly. wikidata should add
        // non-numeric identifiers such as instance_of instead of wdt:P31
        let parsed: Vec<serde_json::Value> = serde_json::from_str(data)?;

        let mut tags = Vec::new();

        for matching in parsed {
            if let Some(v) = matching["categoryLabel"]["value"].as_str() {
                tags.add("wikidata-category", v)
            }
        }
        Ok(tags)
    }
}
