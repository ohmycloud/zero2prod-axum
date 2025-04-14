#!/bin/bash

psql -U postgres -t -c "SELECT datname FROM pg_database WHERE datname like '%-%'" | while read -r dbname; do
    if [ -n "$dbname" ]; then
        echo "Dropping database: $dbname"
        psql -U postgres -c "DROP DATABASE \"$dbname\" WITH (FORCE);"
    fi
done
