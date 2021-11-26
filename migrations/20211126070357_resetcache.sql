delete from extracted.extracted_current;

create view extracted_chunks_view as 
    select e.rowid as rowid, timechunk, tags.text as tag, tag_values.text as value, duration_ms
        from extracted_chunks e
        join tags on tags.id = e.tag
        join tag_values on tag_values.id = e.value;