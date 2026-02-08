INSERT OR REPLACE INTO events(event_id,
                   previous_event_id,
                   event_type_id,
                   event_filename,
                   counter)
VALUES (?1, ?2, ?3, ?4, ?5)