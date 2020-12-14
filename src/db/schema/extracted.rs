table! {
    extracted_events (rowid) {
        rowid -> Nullable<BigInt>,
        timestamp -> BigInt,
        event_id -> Text,
        tag -> Text,
        value -> Text,
    }
}
