CREATE TABLE IF NOT EXISTS events
(
    event_id          INTEGER PRIMARY KEY,
    required_event_id INTEGER REFERENCES events(event_id) ON DELETE SET NULL,
    event_type_id     INTEGER,
    event_filename    TEXT,
    counter           INTEGER
)
