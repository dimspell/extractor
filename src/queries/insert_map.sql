INSERT OR REPLACE INTO maps(id,
                 map_filename,
                 map_name,
                 pgp_filename,
                 dlg_filename,
                 is_light)
VALUES (?1,
        ?2,
        ?3,
        ?4,
        ?5,
        ?6)