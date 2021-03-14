-- Your SQL goes here
CREATE TABLE IF NOT EXISTS extracted.tags (
    id integer PRIMARY KEY NOT NULL,
    text text NOT NULL
);

CREATE TABLE IF NOT EXISTS extracted.tag_values (
    id integer PRIMARY KEY NOT NULL,
    text text NOT NULL
);

CREATE TABLE IF NOT EXISTS extracted.event_ids (
    id integer PRIMARY KEY NOT NULL,
    raw_id text NOT NULL,
    timestamp_unix_ms bigint NOT NULL,
    duration_ms bigint NOT NULL
);

CREATE TABLE IF NOT EXISTS extracted.extracted_events (
    rowid integer PRIMARY KEY NOT NULL,
    event_id bigint NOT NULL REFERENCES event_ids (id),
    timestamp_unix_ms bigint NOT NULL,
    duration_ms bigint NOT NULL,
    tag bigint NOT NULL REFERENCES tags (id),
    value bigint NOT NULL REFERENCES tag_values (id)
);

CREATE INDEX IF NOT EXISTS extracted.ee_timestamp_idx ON extracted.extracted_events (timestamp_unix_ms);

CREATE INDEX IF NOT EXISTS extracted.ee_tag_timestamp_idx ON extracted.extracted_events (tag, timestamp_unix_ms);

CREATE TABLE IF NOT EXISTS extracted.extracted_current (
    utc_date text PRIMARY KEY NOT NULL,
    extracted_timestamp_unix_ms bigint NOT NULL,
    raw_events_changed_timestamp_unix_ms bigint NOT NULL
);

CREATE TABLE IF NOT EXISTS extracted.fetcher_cache (
    key text PRIMARY KEY NOT NULL,
    timestamp_unix_ms bigint NOT NULL,
    value text NOT NULL
);

