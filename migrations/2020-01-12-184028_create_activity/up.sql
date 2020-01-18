CREATE TABLE activity (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL, -- ISO8601
    data_type TEXT NOT NULL,
    data_type_version INTEGER NOT NULL,
    sampler TEXT NOT NULL, -- JSON
    sampler_sequence_id TEXT NOT NULL, -- UUID
    import_id TEXT, -- any id to prevent duplicate imports
    data TEXT NOT NULL, -- JSON
    UNIQUE(data_type, import_id) ON CONFLICT IGNORE
)

CREATE INDEX activity_timestamp_idx on activity(timestamp);