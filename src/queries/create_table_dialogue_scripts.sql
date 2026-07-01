CREATE TABLE IF NOT EXISTS dialogue_scripts(
    dialog_file TEXT NOT NULL,
    id INTEGER NOT NULL,
    required_event_id INTEGER,
    next_dialog_to_check INTEGER,
    dialog_type_id INTEGER,
    dialog_owner INTEGER,
    dialog_id INTEGER,
    next_dialog_id1 INTEGER,
    next_dialog_id2 INTEGER,
    next_dialog_id3 INTEGER,
    triggered_event_id INTEGER,
    PRIMARY KEY (dialog_file, id),
    FOREIGN KEY (dialog_file, next_dialog_id1) REFERENCES dialogue_scripts(dialog_file, id),
    FOREIGN KEY (dialog_file, next_dialog_id2) REFERENCES dialogue_scripts(dialog_file, id),
    FOREIGN KEY (dialog_file, next_dialog_id3) REFERENCES dialogue_scripts(dialog_file, id)
)
