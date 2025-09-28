use std::time::Duration;

use super::{tags::Tags, ExternalFetcher};
use crate::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;

pub struct YoutubeFetcher;

#[async_trait]
impl ExternalFetcher for YoutubeFetcher {
    fn get_id(&self) -> &'static str {
        "youtube-meta-json"
    }
    fn get_regexes(&self) -> &[TagValueRegex] {
        lazy_static! {
            static ref REGEXES: Vec<TagValueRegex> = vec![TagValueRegex {
                tag: "browse-main-domain".to_string(),
                regex: Regex::new(r#"^youtube.com$"#).unwrap()
            }];
        }

        &REGEXES
    }
    fn get_possible_output_tags(&self) -> &[&str] {
        &[
            "video-title",
            "youtube-uploader",
            "youtube-uploader-name",
            "youtube-channel",
            "youtube-channel-name",
            "youtube-tag",
            "youtube-category",
        ]
    }

    fn get_cache_key(&self, _found: &[regex::Captures], tags: &Tags) -> Option<String> {
        // https://github.com/ytdl-org/youtube-dl/blob/1fb034d029c8b7feafe45f64e6a0808663ad315e/youtube_dl/extractor/youtube.py
        lazy_static! {
            static ref WATCH_REGEX: Regex = Regex::new(
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
        for url in tags.get_all_values_of("browse-url") {
            if let Some(matches) = WATCH_REGEX.captures(url) {
                let id = matches.name("id").unwrap().as_str();
                log::trace!("url={}, id={}", url, id);
                return Some(id.to_string());
            }
        }
        None
    }

    async fn fetch_data(&self, cache_key: &str) -> Result<String, FetchError> {
        log::debug!("querying youtube for {}", cache_key);
        let data =
            youtube_dl::YoutubeDl::new(format!("https://www.youtube.com/watch?v={cache_key}"))
                .run()
                .map_err(|e| match &e {
                    youtube_dl::Error::ExitCode { code: _, stderr }
                        if stderr.contains("Video unavailable")
                            && (stderr.contains("copyright claim")
                                || stderr.contains(
                                    "account associated with this video has been terminated",
                                )) =>
                    {
                        FetchError::PermanentFailure(Box::new(e))
                    }
                    youtube_dl::Error::Io(_)
                    | youtube_dl::Error::Json(_)
                    | youtube_dl::Error::ProcessTimeout
                    | _ => FetchError::TemporaryFailure(Box::new(e), Duration::from_secs(60)),
                })?;
        serde_json::to_string(&data)
            .context("serializing ytdl output")
            .map_err(|e| FetchError::TemporaryFailure(e.into(), Duration::from_secs(1)))
    }

    async fn process_data(
        &self,
        _tags: &Tags,
        _cache_key: &str,
        data: &str,
    ) -> anyhow::Result<Vec<TagValue>> {
        use youtube_dl::*;
        let d: YoutubeDlOutput = serde_json::from_str(data).context("serde")?;
        let mut tags: Vec<TagValue> = Vec::new();
        if let YoutubeDlOutput::SingleVideo(sv) = d {
            if let Some(u) = sv.title {
                tags.add("video-title", u);
            }
            if let Some(u) = sv.uploader_id {
                tags.add("youtube-uploader", u)
            }
            if let Some(u) = sv.uploader {
                tags.add("youtube-uploader-name", u)
            }
            if let Some(u) = sv.channel_id {
                tags.add("youtube-channel", u)
            }
            if let Some(u) = sv.channel {
                tags.add("youtube-channel-name", u)
            }
            for tag in sv.tags.into_iter().flatten().flatten() {
                tags.add("youtube-tag", tag);
            }
            if let Some(tg) = sv.categories {
                for tag in tg.into_iter().flatten() {
                    tags.add("youtube-category", tag);
                }
            }
        } else {
            anyhow::bail!("got playlist??");
        }
        Ok(tags)
    }
}
