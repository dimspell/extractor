CREATE TABLE IF NOT EXISTS events
(
    event_id          INTEGER,
    required_event_id INTEGER,
    event_type_id     INTEGER,
    event_filename    TEXT,
    counter           INTEGER
)
