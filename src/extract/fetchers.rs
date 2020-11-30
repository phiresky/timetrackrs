use super::tags::Tags;
use crate::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;
pub trait Fetcher {
    fn get_id(&self) -> &'static str;
    fn get_cache_key(&self, tags: &Tags) -> Option<String>;
    fn get_data(&self, cache_key: &str) -> Option<String>;
    fn process_data(&self, tags: &Tags, cache_key: &str, data: &str) -> Tags
}

pub struct YoutubeFetcher;

impl Fetcher for YoutubeFetcher {
    fn get_id(&self) -> &'static str {
        "youtube-meta-json"
    }

    fn get_cache_key(&self, tags: &Tags) -> Option<String> {
        // https://github.com/ytdl-org/youtube-dl/blob/1fb034d029c8b7feafe45f64e6a0808663ad315e/youtube_dl/extractor/youtube.py
        lazy_static! {
            static ref watch_regex: Regex = Regex::new(
                r#"(?x)^
                     (
                         (?:https?://|//)                                    # http(s):// or protocol-independent URL
                         (?:(?:(?:(?:\w+\.)?[yY][oO][uU][tT][uU][bB][eE](?:-nocookie|kids)?\.com/|
                            (?:www\.)?deturl\.com/www\.youtube\.com/|
                            (?:www\.)?pwnyoutube\.com/|
                            (?:www\.)?hooktube\.com/|
                            (?:www\.)?yourepeat\.com/|
                            tube\.majestyc\.net/|
                            # Invidious instances taken from https://github.com/omarroth/invidious/wiki/Invidious-Instances
                            (?:(?:www|dev)\.)?invidio\.us/|
                            (?:(?:www|no)\.)?invidiou\.sh/|
                            (?:(?:www|fi|de)\.)?invidious\.snopyta\.org/|
                            (?:www\.)?invidious\.kabi\.tk/|
                            (?:www\.)?invidious\.13ad\.de/|
                            (?:www\.)?invidious\.mastodon\.host/|
                            (?:www\.)?invidious\.nixnet\.xyz/|
                            (?:www\.)?invidious\.drycat\.fr/|
                            (?:www\.)?tube\.poal\.co/|
                            (?:www\.)?vid\.wxzm\.sx/|
                            (?:www\.)?yewtu\.be/|
                            (?:www\.)?yt\.elukerio\.org/|
                            (?:www\.)?yt\.lelux\.fi/|
                            (?:www\.)?invidious\.ggc-project\.de/|
                            (?:www\.)?yt\.maisputain\.ovh/|
                            (?:www\.)?invidious\.13ad\.de/|
                            (?:www\.)?invidious\.toot\.koeln/|
                            (?:www\.)?invidious\.fdn\.fr/|
                            (?:www\.)?watch\.nettohikari\.com/|
                            (?:www\.)?kgg2m7yk5aybusll\.onion/|
                            (?:www\.)?qklhadlycap4cnod\.onion/|
                            (?:www\.)?axqzx4s6s54s32yentfqojs3x5i7faxza6xo3ehd4bzzsg2ii4fv2iid\.onion/|
                            (?:www\.)?c7hqkpkpemu6e7emz5b4vyz7idjgdvgaaa3dyimmeojqbgpea3xqjoid\.onion/|
                            (?:www\.)?fz253lmuao3strwbfbmx46yu7acac2jz27iwtorgmbqlkurlclmancad\.onion/|
                            (?:www\.)?invidious\.l4qlywnpwqsluw65ts7md3khrivpirse744un3x7mlskqauz5pyuzgqd\.onion/|
                            (?:www\.)?owxfohz4kjyv25fvlqilyxast7inivgiktls3th44jhk3ej3i7ya\.b32\.i2p/|
                            (?:www\.)?4l2dgddgsrkf2ous66i6seeyi6etzfgrue332grh2n7madpwopotugyd\.onion/|
                            youtube\.googleapis\.com/)                        # the various hostnames, with wildcard subdomains
                         (?:.*?\#/)?                                          # handle anchor (#/) redirect urls
                         (?:                                                  # the various things that can precede the ID:
                             (?:(?:v|embed|e)/)                # v/ or embed/ or e/
                             |(?:                                             # or the v= param in all its forms
                                 (?:(?:watch|movie)(?:_popup)?(?:\.php)?/?)?  # preceding watch(_popup|.php) or nothing (like /?v=xxxx)
                                 (?:\?|\#!?)                                  # the params delimiter ? or # or #!
                                 (?:.*?[&;])??                                # any other preceding param (like /?s=tuff&v=xxxx or ?s=tuff&amp;v=V36LpHqtcDY)
                                 v=
                             )
                         ))
                         |(?:
                            youtu\.be|                                        # just youtu.be/xxxx
                            vid\.plus|                                        # or vid.plus/xxxx
                            zwearz\.com/watch|                                # or zwearz.com/watch/xxxx
                         )/
                         |(?:www\.)?cleanvideosearch\.com/media/action/yt/watch\?videoId=
                         )
                     )
                     (?P<id>[0-9A-Za-z_-]{11})                                      # here is it! the YouTube video ID
                     .*                                                # if we found the ID, everything can follow
                     $"#
            ).unwrap();
        }
        for tag in tags {
            if tag.starts_with("browse-url:") {
                let url = &tag["browse-url:".len()..];
                if let Some(matches) = watch_regex.captures(url) {
                    let id = matches.name("id").unwrap().as_str();
                    log::debug!("url={}, id={}", url, id);
                    return Some(id.to_string());
                }
            }
        }
        None
    }
}
