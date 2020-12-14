#!/bin/bash

dir="$(dirname $0)"

for db in raw_events config extracted; do
    dbfile=$(mktemp)
    diesel migration run --database-url "$dbfile" --migration-dir "$dir/${db}_migrations"
    diesel print-schema --database-url "$dbfile" | sed 's/Integer/BigInt/g' > "$dir/../src/db/schema/$db.rs"
    rm "$dbfile"
done