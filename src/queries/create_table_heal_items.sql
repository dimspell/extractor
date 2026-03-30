CREATE TABLE IF NOT EXISTS heal_items
(
    id             INTEGER,
    name           TEXT,
    description    TEXT,
    base_price     INTEGER,
    pz             INTEGER,
    pm             INTEGER,
    restore_full_health        BOOLEAN,
    restore_full_mana        BOOLEAN,
    poison_heal    BOOLEAN,
    petrif_heal    BOOLEAN,
    polimorph_heal BOOLEAN
)
