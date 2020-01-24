CREATE TABLE events (
    -- for captured events, the id is generated randomly (uuidv4)
    -- for imported events, the id must be an id taken based on
    -- a combination of data_type and something from the import
    -- such that it is unique but repeated imports will not cause
    -- duplicate entries
    id TEXT NOT NULL PRIMARY KEY,
    timestamp TEXT NOT NULL, -- ISO8601
    data_type TEXT NOT NULL, -- "{name}_v{version}"
    sampler TEXT NOT NULL, -- JSON
    sampler_sequence_id TEXT NOT NULL, -- UUID
    data TEXT NOT NULL -- JSON
);

CREATE INDEX events_timestamp_idx on events(timestamp);