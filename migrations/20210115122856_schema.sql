ALTER TABLE raw_events.events RENAME TO raw_events.events_backup_before_2021_01;

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

CREATE INDEX events_timestamp_unix_ms_idx ON raw_events.events (timestamp_unix_ms);

INSERT INTO raw_events.events
SELECT
    insertion_sequence,
    id,
    cast(round((julianday (timestamp) - 2440587.5) * 86400.0 * 1000) AS int) AS timestamp_unix_ms,
    data_type,
    cast(round(coalesce(json_extract (sampler, '$.avg_time'), json_extract (sampler, '$.duration')) * 1000) AS int) AS duration_ms,
    data
FROM
    events_backup_before_2021_01;

-- update events set timestamp_unix_ms = timestamp_unix_ms - timestamp_unix_ms%duration_ms where insertion_sequence <= 848066 and data_type = 'x11_v2'
