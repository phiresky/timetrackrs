#!/bin/bash

dir="$(dirname $0)"

rm -rf "$dir/.dev-db/"
cd "$dir"
mkdir .dev-db
for db in raw_events config extracted; do
    echo "migrating $db"
    dbfile=".dev-db/$db.sqlite3"
    touch "$dbfile"
    DATABASE_URL="sqlite://$dbfile" sqlx migrate --source "diesel/${db}_migrations" run
done