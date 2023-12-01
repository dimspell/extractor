INSERT INTO dialogs(
    id,
    previous_event_id,
    next_dialog_to_check,
    dialog_type_id,
    dialog_owner,
    dialog_id,
    event_id
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)