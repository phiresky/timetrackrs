-- Your SQL goes here
create table extracted_events (
    rowid integer primary key,
    timestamp bigint not null,
    event_id text not null,
    tag text not null,
    value text not null
);

create index ee_event_id_idx on extracted_events (event_id);
create index ee_timestamp_idx on extracted_events (timestamp);
create index ee_tag_timestamp_idx on extracted_events (tag, timestamp);