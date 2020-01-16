table! {
    activity (id) {
        id -> BigInt,
        timestamp -> Text,
        data_type -> Text,
        data_type_version -> Integer,
        sampler -> Text,
        sampler_sequence_id -> Text,
        data -> Text,
    }
}
