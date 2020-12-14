table! {
    events (insertion_sequence) {
        insertion_sequence -> BigInt,
        id -> Text,
        timestamp -> Text,
        data_type -> Text,
        sampler -> Text,
        sampler_sequence_id -> Text,
        data -> Text,
    }
}
