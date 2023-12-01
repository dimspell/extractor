CREATE TABLE IF NOT EXISTS heal_items
(
    id             INTEGER,
    name           TEXT,
    description    TEXT,
    base_price     INTEGER,
    pz             INTEGER,
    pm             INTEGER,
    full_pz        BOOLEAN,
    full_pm        BOOLEAN,
    poison_heal    BOOLEAN,
    petrif_heal    BOOLEAN,
    polimorph_heal BOOLEAN
)