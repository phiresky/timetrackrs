CREATE TABLE activity (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL, -- ISO8601
    data_type TEXT NOT NULL,
    data_type_version INTEGER NOT NULL,
    sampler TEXT NOT NULL, -- JSON
    data TEXT NOT NULL -- JSON
)