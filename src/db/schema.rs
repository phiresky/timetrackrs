table! {
    events (id) {
        id -> Text,
        timestamp -> Text,
        data_type -> Text,
        sampler -> Text,
        sampler_sequence_id -> Text,
        data -> Text,
    }
}

table! {
    fetcher_cache (key) {
        key -> Text,
        timestamp -> Text,
        value -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    events,
    fetcher_cache,
);
