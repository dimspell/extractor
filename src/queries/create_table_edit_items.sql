CREATE TABLE IF NOT EXISTS edit_items
(
    "index"                    INTEGER,
    name                  TEXT,
    description           TEXT,
    base_price            INTEGER,
    runtime_item_id       INTEGER,
    health_points         INTEGER,
    mana_points           INTEGER,
    strength              INTEGER,
    agility               INTEGER,
    wisdom                INTEGER,
    constitution          INTEGER,
    to_dodge              INTEGER,
    to_hit                INTEGER,
    offense               INTEGER,
    defense               INTEGER,
    magical_power         INTEGER,
    modification_resistance INTEGER,
    reserved_byte         INTEGER,
    modifies_item         BOOLEAN,
    additional_effect     INTEGER
)
