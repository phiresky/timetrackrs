CREATE INDEX extracted.ec_timestamp_idx ON extracted_chunks (timechunk);

CREATE INDEX extracted.ec_tag_timestamp_idx ON extracted_chunks (tag, timechunk);

