-- Your SQL goes here
CREATE TABLE IF NOT EXISTS extracted_events (
    rowid integer PRIMARY KEY NOT NULL,
    timestamp text NOT NULL,
    duration double NOT NULL,
    event_id text NOT NULL,
    tag text NOT NULL,
    value text NOT NULL
);

CREATE INDEX IF NOT EXISTS ee_event_id_idx ON extracted_events (event_id);

CREATE INDEX IF NOT EXISTS ee_timestamp_idx ON extracted_events (timestamp);

CREATE INDEX IF NOT EXISTS ee_tag_timestamp_idx ON extracted_events (tag, timestamp);

CREATE TABLE IF NOT EXISTS fetcher_cache (
    key text NOT NULL PRIMARY KEY,
    timestamp text NOT NULL,
    value text NOT NULL
);

CREATE TABLE IF NOT EXISTS extracted_current (
    utc_date text PRIMARY KEY NOT NULL,
    extracted_timestamp text NOT NULL,
    raw_events_changed_timestamp text NOT NULL
);

