DROP TABLE extracted.extracted_events;

CREATE TABLE extracted.extracted_chunks (
    rowid integer PRIMARY KEY NOT NULL,
    timechunk text NOT NULL,
    tag bigint NOT NULL REFERENCES tags (id),
    value bigint NOT NULL REFERENCES tag_values (id),
    duration_ms bigint NOT NULL
);

