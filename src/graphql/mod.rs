use lazy_static::lazy_static;
use reqwest::Client;

type Bigint = i64;
type Json = String;
type Text = String;

mod tracker_events;

lazy_static! {
    static ref CLIENT: Client = Client::new();
}

pub use tracker_events::*;
