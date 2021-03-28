CREATE TABLE raw_events.events (
    -- declared autoincrement to prevent id reuse for synchronization (otherwise sqlite will reuse old ids on e.g. vacuum)
    insertion_sequence integer PRIMARY KEY AUTOINCREMENT NOT NULL,
    -- for captured events, the id is generated randomly (uuidv4)
    -- for imported events, the id must be an id taken based on
    -- a combination of data_type and something from the import
    -- such that it is unique but repeated imports will not cause
    -- duplicate entries
    id text NOT NULL UNIQUE,
    timestamp_unix_ms integer NOT NULL, -- ISO8601
    data_type text NOT NULL, -- "{name}_v{version}"
    duration_ms integer NOT NULL,
    data text NOT NULL -- JSON
);

CREATE INDEX raw_events.events_timestamp_unix_ms_idx ON events (timestamp_unix_ms);

CREATE TABLE config.tag_rule_groups (
    global_id text PRIMARY KEY NOT NULL,
    data text NOT NULL
);

CREATE TABLE extracted.tags (
    id integer PRIMARY KEY NOT NULL,
    text text UNIQUE NOT NULL
);

CREATE TABLE extracted.tag_values (
    id integer PRIMARY KEY NOT NULL,
    text text UNIQUE NOT NULL
);

CREATE TABLE extracted.event_ids (
    id integer PRIMARY KEY NOT NULL,
    raw_id text UNIQUE NOT NULL,
    timestamp_unix_ms bigint NOT NULL,
    duration_ms bigint NOT NULL
);

CREATE TABLE extracted.extracted_events (
    rowid integer PRIMARY KEY NOT NULL,
    event_id bigint NOT NULL REFERENCES event_ids (id),
    timestamp_unix_ms bigint NOT NULL,
    duration_ms bigint NOT NULL,
    tag bigint NOT NULL REFERENCES tags (id),
    value bigint NOT NULL REFERENCES tag_values (id)
);

CREATE INDEX extracted.ee_timestamp_idx ON extracted_events (timestamp_unix_ms);

CREATE INDEX extracted.ee_tag_timestamp_idx ON extracted_events (tag, timestamp_unix_ms);

CREATE TABLE extracted.extracted_current (
    utc_date text PRIMARY KEY NOT NULL,
    extracted_timestamp_unix_ms bigint NOT NULL,
    raw_events_changed_timestamp_unix_ms bigint NOT NULL
);

CREATE TABLE extracted.fetcher_cache (
    key text PRIMARY KEY NOT NULL,
    timestamp_unix_ms bigint NOT NULL,
    value text NOT NULL
);

