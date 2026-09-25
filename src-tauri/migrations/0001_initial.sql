CREATE TABLE usage_events (
    id INTEGER PRIMARY KEY,
    message_id TEXT NOT NULL,
    request_id TEXT NOT NULL,
    ts INTEGER NOT NULL,
    source TEXT NOT NULL,
    model TEXT NOT NULL,
    project TEXT NOT NULL,
    input INTEGER NOT NULL,
    output INTEGER NOT NULL,
    cache_create INTEGER NOT NULL,
    cache_read INTEGER NOT NULL,
    UNIQUE (message_id, request_id)
) STRICT;

CREATE INDEX usage_events_ts ON usage_events (ts);

CREATE TABLE limit_snapshots (
    id INTEGER PRIMARY KEY,
    ts INTEGER NOT NULL,
    window_kind TEXT NOT NULL,
    utilization REAL NOT NULL,
    resets_at INTEGER
) STRICT;

CREATE INDEX limit_snapshots_kind_ts ON limit_snapshots (window_kind, ts);

CREATE TABLE file_offsets (
    path TEXT PRIMARY KEY,
    byte_offset INTEGER NOT NULL
) STRICT;

CREATE TABLE settings (
    name TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT;
