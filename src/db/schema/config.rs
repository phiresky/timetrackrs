table! {
    fetcher_cache (key) {
        key -> Text,
        timestamp -> Text,
        value -> Text,
    }
}

table! {
    tag_rule_groups (global_id) {
        global_id -> Text,
        data -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    fetcher_cache,
    tag_rule_groups,
);
