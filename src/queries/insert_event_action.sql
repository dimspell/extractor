INSERT OR REPLACE INTO event_actions(event_id,
                   action_order,
                   action_prefix,
                   function_name,
                   parameters,
                   raw_content)
VALUES (?1,
        ?2,
        ?3,
        ?4,
        ?5,
        ?6)
