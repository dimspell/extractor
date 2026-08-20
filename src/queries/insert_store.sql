INSERT OR REPLACE INTO stores(id,
                   store_name,
                   inn_night_cost,
                   price_modifier,
                   invitation,
                   haggle_success,
                   haggle_fail)
VALUES (?1,
        ?2,
        ?3,
        ?4,
        ?5,
        ?6,
        ?7)