use super::ExternalFetcher;
use crate::{expand::get_capture, prelude::*};
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
impl ExternalFetcher for WikidataIdFetcher {
    fn get_id(&self) -> &'static str {
        "wikidata-domain-to-id-v1"
    }

    fn get_regexes(&self) -> &[Regex] {
        lazy_static::lazy_static! {
            static ref regexes: Vec<Regex> =
                vec![Regex::new(r#"^browse-full-domain:(?P<domain>.*)$"#).unwrap()];
        }
        &regexes
    }

    fn get_cache_key(
        &self,
        found: &[regex::Captures],
        tags: &crate::prelude::Tags,
    ) -> Option<String> {
        get_capture(found, "domain").map(|d| d.to_string())
    }

    fn fetch_data(&self, cache_key: &str) -> anyhow::Result<String> {
        log::debug!("fetching {} from wikidata", cache_key);
        let api = mediawiki::api_sync::ApiSync::new("https://www.wikidata.org/w/api.php")
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("wikidata api initialization")?; // Will determine the SPARQL API URL via site info data

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
        let main_domain_urls: Vec<String> = super::public_suffixes
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
            .unwrap_or(vec![]);

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
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        let (full_domain_matches, main_domain_matches): (
            Vec<&serde_json::Value>,
            Vec<&serde_json::Value>,
        ) = mutres["results"]["bindings"]
            .as_array()
            .context("unparseable response")?
            .into_iter()
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

    fn process_data(
        &self,
        tags: &crate::prelude::Tags,
        cache_key: &str,
        data: &str,
    ) -> anyhow::Result<crate::prelude::Tags> {
        let parsed: serde_json::Value = serde_json::from_str(data)?;

        let mut tags = Tags::new();

        let matches = parsed["full_domain_matches"]
            .as_array()
            .filter(|e| e.len() > 0)
            .or(parsed["main_domain_matches"].as_array())
            .context("internal error?")?;

        for matching in matches {
            matching["website_url"]["value"]
                .as_str()
                .map(|e| tags.insert(format!("wikidata-website-url:{}", e)));
            matching["service"]["value"]
                .as_str()
                .map(|e| tags.insert(format!("wikidata-id:{}", &e[31..])));

            matching["serviceLabel"]["value"]
                .as_str()
                .map(|e| tags.insert(format!("wikidata-label:{}", e)));
        }
        Ok(tags)
    }
}

pub struct WikidataCategoryFetcher;
impl ExternalFetcher for WikidataCategoryFetcher {
    fn get_id(&self) -> &'static str {
        "wikidata-id-to-class"
    }

    fn get_regexes(&self) -> &[Regex] {
        lazy_static::lazy_static! {
            static ref regexes: Vec<Regex> =
                vec![Regex::new(r#"^wikidata-id:(?P<id>.*)$"#).unwrap()];
        }
        &regexes
    }

    fn get_cache_key(
        &self,
        found: &[regex::Captures],
        tags: &crate::prelude::Tags,
    ) -> Option<String> {
        get_capture(found, "id").map(|d| d.to_string())
    }

    fn fetch_data(&self, cache_key: &str) -> anyhow::Result<String> {
        let api = mediawiki::api_sync::ApiSync::new("https://www.wikidata.org/w/api.php")
            .map_err(|e| anyhow::anyhow!("{:?}", e))
            .context("wikidata api initialization")?; // Will determine the SPARQL API URL via site info data

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
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        Ok(format!("{}", res["results"]["bindings"]))
    }

    fn process_data(
        &self,
        tags: &crate::prelude::Tags,
        cache_key: &str,
        data: &str,
    ) -> anyhow::Result<crate::prelude::Tags> {
        // in theory we should use the ids. but eh that's sooo ugly. wikidata should add
        // non-numeric identifiers such as instance_of instead of wdt:P31
        let parsed: Vec<serde_json::Value> = serde_json::from_str(data)?;

        let mut tags = Tags::new();

        for matching in parsed {
            matching["categoryLabel"]["value"]
                .as_str()
                .map(|v| tags.insert(format!("wikidata-category:{}", v)));
        }
        Ok(tags)
    }
}