CREATE TABLE IF NOT EXISTS dialogue_scripts(
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
