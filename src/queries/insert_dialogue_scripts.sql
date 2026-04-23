INSERT OR REPLACE INTO dialogue_scripts(
    dialog_file,
    id,
    required_event_id,
    next_dialog_to_check,
    dialog_type_id,
    dialog_owner,
    dialog_id,
    next_dialog_id1,
    next_dialog_id2,
    next_dialog_id3,
    triggered_event_id
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
