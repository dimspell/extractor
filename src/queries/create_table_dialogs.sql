CREATE TABLE IF NOT EXISTS dialogs(
    id INTEGER,
    previous_event_id INTEGER,
    next_dialog_to_check INTEGER,
    dialog_type_id INTEGER,
    dialog_owner INTEGER,
    dialog_id INTEGER,
    event_id INTEGER
)