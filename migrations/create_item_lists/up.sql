CREATE TABLE item_lists (
    id INTEGER PRIMARY KEY,
    created TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted INTEGER NOT NULL DEFAULT FALSE,
    folder VARCHAR NOT NULL DEFAULT 'default',
    access VARCHAR NOT NULL DEFAULT 'Public',
    list_type VARCHAR NOT NULL DEFAULT 'Standard',
    name VARCHAR NOT NULL,
    modified TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
)