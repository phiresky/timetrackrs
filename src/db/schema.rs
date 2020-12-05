table! {
    events (insertion_sequence) {
        insertion_sequence -> Integer,
        id -> Text,
        timestamp -> Text,
        data_type -> Text,
        sampler -> Text,
        sampler_sequence_id -> Text,
        data -> Text,
    }
}

table! {
    events_old (id) {
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
    events_old,
    fetcher_cache,
);
