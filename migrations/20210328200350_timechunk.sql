DROP TABLE extracted.extracted_current;

CREATE TABLE extracted.extracted_current (
    timechunk text PRIMARY KEY NOT NULL,
    extracted_timestamp_unix_ms bigint NOT NULL,
    raw_events_changed_timestamp_unix_ms bigint NOT NULL
);

