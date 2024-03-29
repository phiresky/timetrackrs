#!/bin/bash

# the normal sqlx migration CLI sadly doesn't work for us since it only allows a single sqlite db
dir="$(dirname $0)"

rm -rf "$dir/.dev-db/"
cd "$dir"
mkdir .dev-db

for db in raw_events extracted config; do
    script="$script
attach '$db.sqlite3' as $db;
"
done
script="$script
$(cat migrations/*.sql)"
echo "$script"
(
    cd .dev-db
    echo "$script" | sqlite3 lock.sqlite3
)