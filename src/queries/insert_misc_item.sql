INSERT OR REPLACE INTO misc_items(id,
                                 name,
                                 description,
                                 base_price,
                                 reserved_bytes,
                                 runtime_record_index_slot
                                 )
VALUES (?1,
        ?2,
        ?3,
        ?4,
        ?5,
        ?6);
