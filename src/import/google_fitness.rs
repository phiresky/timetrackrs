#![allow(non_snake_case)]

use crate::prelude::*;

use google_fitness1::DataSource;
use google_fitness1::Fitness;
use google_fitness1::{Error, Result};
use std::default::Default;
use structopt::StructOpt;
use yup_oauth2::authenticator::Authenticator;
use yup_oauth2::authenticator::InstalledFlowAuthenticator;
use yup_oauth2::ApplicationSecret;

#[derive(StructOpt)]
pub struct GoogleFitImportArgs {}

#[derive(Debug, Serialize, Deserialize, TypeScriptify)]
pub struct GoogleFitSession {
    // os_info: util::OsInfo,
//event: JournaldEvent,
}

impl ExtractInfo for GoogleFitSession {
    fn extract_info(&self) -> Option<ExtractedInfo> {
        None
    }
}
lazy_static! {
    // example:
    // -74 daee0006297641feb18738955c7c125e Fri 2019-06-21 18:14:04 UTC—Fri 2019-06-21 20:29:58 UTC
    // -73 7754c38d30434a99b25640c1b32c1af3 Sat 2019-06-22 15:48:03 UTC—Thu 2019-06-27 23:13:27 UTC
    static ref JOURNALD_LIST_BOOTS: regex::Regex = regex::Regex::new(
        r#"(?x)
        ^ # start line
        \s*(?P<relative_boot_number>-?\d+)\  # relative boot number
        (?P<boot_id>[0-9a-f]+)\ 
        ...\ # week day
        (?P<start>\d\d\d\d-\d\d-\d\d\ \d\d:\d\d:\d\d)\ UTC
        —...\ (?P<end>\d\d\d\d-\d\d-\d\d\ \d\d:\d\d:\d\d)\ UTC
        $ # end line
        "#
    )
    .unwrap();
}
impl Importable for GoogleFitImportArgs {
    fn import(&self) -> anyhow::Result<Vec<NewDbEvent>> {
        // Get an ApplicationSecret instance by some means. It contains the `client_id` and
        // `client_secret`, among other things.
        let secret: ApplicationSecret = Default::default();
        // Instantiate the authenticator. It will choose a suitable authentication flow for you,
        // unless you replace  `None` with the desired Flow.
        // Provide your own `AuthenticatorDelegate` to adjust the way it operates and get feedback about
        // what's going on. You probably want to bring in your own `TokenStorage` to persist tokens and
        // retrieve them from storage.
        let auth = InstalledFlowAuthenticator::builder(
            secret,
            yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
        )
        .persist_tokens_to_disk("tokenscache.json")
        .build();
        let auth = futures::executor::block_on(auth)?;
        let mut https_client = hyper::Client::builder().build(hyper_tls::HttpsConnector::new());
        let mut hub = Fitness::new(&mut https_client, auth);
        // As the method needs a request, you would usually fill it with the desired information
        // into the respective structure. Some of the parts shown here might not be applicable !
        // Values shown here are possibly random and not representative !
        let mut req = DataSource::default();

        Ok(vec![])
    }
}
