table! {
    use diesel::sql_types::*;
    use crate::db::hack::Integer;

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
    use diesel::sql_types::*;
    use crate::db::hack::Integer;

    fetcher_cache (key) {
        key -> Text,
        timestamp -> Text,
        value -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::hack::Integer;

    tag_rule_groups (global_id) {
        global_id -> Text,
        data -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    events,
    fetcher_cache,
    tag_rule_groups,
);
