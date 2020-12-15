-- Your SQL goes here
CREATE TABLE IF NOT EXISTS tags (
    id integer PRIMARY KEY NOT NULL,
    text text NOT NULL
);

CREATE TABLE IF NOT EXISTS tag_values (
    id integer PRIMARY KEY NOT NULL,
    text text NOT NULL
);

CREATE TABLE IF NOT EXISTS event_ids (
    id integer PRIMARY KEY NOT NULL,
    raw_id text NOT NULL,
    timestamp bigint NOT NULL,
    duration double NOT NULL
);

CREATE TABLE IF NOT EXISTS extracted_events (
    rowid integer PRIMARY KEY NOT NULL,
    event_id bigint NOT NULL REFERENCES event_ids (id),
    timestamp bigint NOT NULL,
    duration double NOT NULL,
    tag bigint NOT NULL REFERENCES tags (id),
    value bigint NOT NULL REFERENCES tag_values (id)
);

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

