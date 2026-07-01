CREATE TABLE IF NOT EXISTS dialogue_scripts(
    dialog_file_id INTEGER NOT NULL REFERENCES dialogue_script_files(id),
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
    PRIMARY KEY (dialog_file_id, id),
    FOREIGN KEY (dialog_file_id, dialog_id) REFERENCES dialogue_paragraphs(file_id, id),
    FOREIGN KEY (dialog_file_id, next_dialog_id1) REFERENCES dialogue_scripts(dialog_file_id, id),
    FOREIGN KEY (dialog_file_id, next_dialog_id2) REFERENCES dialogue_scripts(dialog_file_id, id),
    FOREIGN KEY (dialog_file_id, next_dialog_id3) REFERENCES dialogue_scripts(dialog_file_id, id)
)
