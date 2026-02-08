INSERT OR REPLACE INTO monster_refs(file_path,
                         id,
                         file_id,
                         mon_id,
                         pos_x,
                         pos_y,
                         loot1_item_id,
                         loot1_item_type,
                         loot2_item_id,
                         loot2_item_type,
                         loot3_item_id,
                         loot3_item_type)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12);