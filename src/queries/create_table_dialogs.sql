CREATE TABLE IF NOT EXISTS dialogs(
    dialog_file TEXT,
    id INTEGER,
    required_event_id INTEGER,
    next_dialog_to_check INTEGER,
    dialog_type_id INTEGER,
    dialog_owner INTEGER,
    dialog_id INTEGER,
    next_dialog_id1 INTEGER,
    next_dialog_id2 INTEGER,
    next_dialog_id3 INTEGER,
    triggered_event_id INTEGER
)
