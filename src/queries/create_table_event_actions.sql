CREATE TABLE IF NOT EXISTS event_actions
(
    event_id            INTEGER,
    action_order        INTEGER,
    prefix       TEXT,
    function_name       TEXT,
    parameters          TEXT,
    raw_content         TEXT
)
