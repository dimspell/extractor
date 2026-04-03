use super::types::{
    extract_as, extract_map_file, extract_sprite_info, extract_tileset, patch_as,
    patch_not_supported, validate_as, DetectKind, FileType,
};

pub(crate) fn make_all_map_ini() -> FileType {
    FileType {
        key: "all_maps",
        name: "AllMap.ini",
        description: "Master map list",
        extensions: &[".ini"],
        detect_kind: DetectKind::Ini("AllMap.ini"),
        extract_fn: extract_as::<crate::references::all_map_ini::Map>,
        patch_fn: patch_as::<crate::references::all_map_ini::Map>,
        validate_fn: Some(validate_as::<crate::references::all_map_ini::Map>),
    }
}

pub(crate) fn make_map_ini() -> FileType {
    FileType {
        key: "map_ini",
        name: "Map.ini",
        description: "Map properties",
        extensions: &[".ini"],
        detect_kind: DetectKind::Ini("Map.ini"),
        extract_fn: extract_as::<crate::references::map_ini::MapIni>,
        patch_fn: patch_as::<crate::references::map_ini::MapIni>,
        validate_fn: Some(validate_as::<crate::references::map_ini::MapIni>),
    }
}

pub(crate) fn make_extra_ini() -> FileType {
    FileType {
        key: "extra_ini",
        name: "Extra.ini",
        description: "Interactive object types",
        extensions: &[".ini"],
        detect_kind: DetectKind::Ini("Extra.ini"),
        extract_fn: extract_as::<crate::references::extra_ini::Extra>,
        patch_fn: patch_as::<crate::references::extra_ini::Extra>,
        validate_fn: Some(validate_as::<crate::references::extra_ini::Extra>),
    }
}

pub(crate) fn make_event_ini() -> FileType {
    FileType {
        key: "event_ini",
        name: "Event.ini",
        description: "Script/event mappings",
        extensions: &[".ini"],
        detect_kind: DetectKind::Ini("Event.ini"),
        extract_fn: extract_as::<crate::references::event_ini::Event>,
        patch_fn: patch_as::<crate::references::event_ini::Event>,
        validate_fn: Some(validate_as::<crate::references::event_ini::Event>),
    }
}

pub(crate) fn make_monster_ini() -> FileType {
    FileType {
        key: "monster_ini",
        name: "Monster.ini",
        description: "Monster visual refs",
        extensions: &[".ini"],
        detect_kind: DetectKind::Ini("Monster.ini"),
        extract_fn: extract_as::<crate::references::monster_ini::MonsterIni>,
        patch_fn: patch_as::<crate::references::monster_ini::MonsterIni>,
        validate_fn: Some(validate_as::<crate::references::monster_ini::MonsterIni>),
    }
}

pub(crate) fn make_npc_ini() -> FileType {
    FileType {
        key: "npc_ini",
        name: "Npc.ini",
        description: "NPC visual refs",
        extensions: &[".ini"],
        detect_kind: DetectKind::Ini("Npc.ini"),
        extract_fn: extract_as::<crate::references::npc_ini::NpcIni>,
        patch_fn: patch_as::<crate::references::npc_ini::NpcIni>,
        validate_fn: Some(validate_as::<crate::references::npc_ini::NpcIni>),
    }
}

pub(crate) fn make_wave_ini() -> FileType {
    FileType {
        key: "wave_ini",
        name: "Wave.ini",
        description: "Audio/SNF references",
        extensions: &[".ini"],
        detect_kind: DetectKind::Ini("Wave.ini"),
        extract_fn: extract_as::<crate::references::wave_ini::WaveIni>,
        patch_fn: patch_as::<crate::references::wave_ini::WaveIni>,
        validate_fn: Some(validate_as::<crate::references::wave_ini::WaveIni>),
    }
}

pub(crate) fn make_weapons() -> FileType {
    FileType {
        key: "weapons",
        name: "WeaponItem.db",
        description: "Weapons & armor database",
        extensions: &[".db"],
        detect_kind: DetectKind::Db(&["WeaponItem.db", "weaponItem.db"]),
        extract_fn: extract_as::<crate::references::weapons_db::WeaponItem>,
        patch_fn: patch_as::<crate::references::weapons_db::WeaponItem>,
        validate_fn: Some(validate_as::<crate::references::weapons_db::WeaponItem>),
    }
}

pub(crate) fn make_monsters() -> FileType {
    FileType {
        key: "monsters",
        name: "Monster.db",
        description: "Monster attributes",
        extensions: &[".db"],
        detect_kind: DetectKind::Db(&["Monster.db", "monster.db"]),
        extract_fn: extract_as::<crate::references::monster_db::Monster>,
        patch_fn: patch_as::<crate::references::monster_db::Monster>,
        validate_fn: Some(validate_as::<crate::references::monster_db::Monster>),
    }
}

pub(crate) fn make_magic() -> FileType {
    FileType {
        key: "magic",
        name: "Magic.db",
        description: "Magic spell records",
        extensions: &[".db"],
        detect_kind: DetectKind::Db(&["Magic.db", "magic.db", "MulMagic.db"]),
        extract_fn: extract_as::<crate::references::magic_db::MagicSpell>,
        patch_fn: patch_as::<crate::references::magic_db::MagicSpell>,
        validate_fn: None,
    }
}

pub(crate) fn make_store() -> FileType {
    FileType {
        key: "store",
        name: "Store.db",
        description: "Shop inventory records",
        extensions: &[".db"],
        detect_kind: DetectKind::Db(&["Store.db", "STORE.DB", "store.db"]),
        extract_fn: extract_as::<crate::references::store_db::Store>,
        patch_fn: patch_as::<crate::references::store_db::Store>,
        validate_fn: None,
    }
}

pub(crate) fn make_misc_item() -> FileType {
    FileType {
        key: "misc_item",
        name: "MiscItem.db",
        description: "Generic item records",
        extensions: &[".db"],
        detect_kind: DetectKind::Db(&["MiscItem.db", "miscitem.db"]),
        extract_fn: extract_as::<crate::references::misc_item_db::MiscItem>,
        patch_fn: patch_as::<crate::references::misc_item_db::MiscItem>,
        validate_fn: Some(validate_as::<crate::references::misc_item_db::MiscItem>),
    }
}

pub(crate) fn make_heal_item() -> FileType {
    FileType {
        key: "heal_item",
        name: "HealItem.db",
        description: "Consumable records",
        extensions: &[".db"],
        detect_kind: DetectKind::Db(&["HealItem.db", "healitem.db"]),
        extract_fn: extract_as::<crate::references::heal_item_db::HealItem>,
        patch_fn: patch_as::<crate::references::heal_item_db::HealItem>,
        validate_fn: Some(validate_as::<crate::references::heal_item_db::HealItem>),
    }
}

pub(crate) fn make_event_item() -> FileType {
    FileType {
        key: "event_item",
        name: "EventItem.db",
        description: "Quest item records",
        extensions: &[".db"],
        detect_kind: DetectKind::Db(&["EventItem.db", "eventitem.db"]),
        extract_fn: extract_as::<crate::references::event_item_db::EventItem>,
        patch_fn: patch_as::<crate::references::event_item_db::EventItem>,
        validate_fn: Some(validate_as::<crate::references::event_item_db::EventItem>),
    }
}

pub(crate) fn make_edit_item() -> FileType {
    FileType {
        key: "edit_item",
        name: "EditItem.db",
        description: "Modifiable item records",
        extensions: &[".db"],
        detect_kind: DetectKind::Db(&["EditItem.db", "edititem.db"]),
        extract_fn: extract_as::<crate::references::edit_item_db::EditItem>,
        patch_fn: patch_as::<crate::references::edit_item_db::EditItem>,
        validate_fn: Some(validate_as::<crate::references::edit_item_db::EditItem>),
    }
}

pub(crate) fn make_party_level() -> FileType {
    FileType {
        key: "party_level",
        name: "PrtLevel.db",
        description: "EXP table records",
        extensions: &[".db"],
        detect_kind: DetectKind::Db(&["PrtLevel.db", "prtlevel.db"]),
        extract_fn: extract_as::<crate::references::party_level_db::PartyLevelNpc>,
        patch_fn: patch_as::<crate::references::party_level_db::PartyLevelNpc>,
        validate_fn: None,
    }
}

pub(crate) fn make_party_ini() -> FileType {
    FileType {
        key: "party_ini",
        name: "PrtIni.db",
        description: "Party NPC metadata",
        extensions: &[".db"],
        detect_kind: DetectKind::Db(&["PrtIni.db", "prtini.db"]),
        extract_fn: extract_as::<crate::references::party_ini_db::PartyIniNpc>,
        patch_fn: patch_as::<crate::references::party_ini_db::PartyIniNpc>,
        validate_fn: None,
    }
}

pub(crate) fn make_chdata() -> FileType {
    FileType {
        key: "chdata",
        name: "ChData.db",
        description: "Character data records",
        extensions: &[".db"],
        detect_kind: DetectKind::Db(&["ChData.db", "chdata.db"]),
        extract_fn: extract_as::<crate::references::chdata_db::ChData>,
        patch_fn: patch_as::<crate::references::chdata_db::ChData>,
        validate_fn: None,
    }
}

pub(crate) fn make_party_ref() -> FileType {
    FileType {
        key: "party_ref",
        name: "PartyRef.ref",
        description: "Character definitions",
        extensions: &[".ref"],
        detect_kind: DetectKind::RefPrefix("PartyRef"),
        extract_fn: extract_as::<crate::references::party_ref::PartyRef>,
        patch_fn: patch_as::<crate::references::party_ref::PartyRef>,
        validate_fn: Some(validate_as::<crate::references::party_ref::PartyRef>),
    }
}

pub(crate) fn make_draw_item() -> FileType {
    FileType {
        key: "draw_item",
        name: "DRAWITEM.ref",
        description: "Map placement records",
        extensions: &[".ref"],
        detect_kind: DetectKind::RefPrefix("DRAWITEM"),
        extract_fn: extract_as::<crate::references::draw_item::DrawItem>,
        patch_fn: patch_as::<crate::references::draw_item::DrawItem>,
        validate_fn: Some(validate_as::<crate::references::draw_item::DrawItem>),
    }
}

pub(crate) fn make_npc_ref() -> FileType {
    FileType {
        key: "npc_ref",
        name: "Npc*.ref",
        description: "NPC placement records",
        extensions: &[".ref"],
        detect_kind: DetectKind::RefPrefix("Npc"),
        extract_fn: extract_as::<crate::references::npc_ref::NPC>,
        patch_fn: patch_as::<crate::references::npc_ref::NPC>,
        validate_fn: None,
    }
}

pub(crate) fn make_monster_ref() -> FileType {
    FileType {
        key: "monster_ref",
        name: "Mon*.ref",
        description: "Monster placement records",
        extensions: &[".ref"],
        detect_kind: DetectKind::RefPrefix("Mon"),
        extract_fn: extract_as::<crate::references::monster_ref::MonsterRef>,
        patch_fn: patch_as::<crate::references::monster_ref::MonsterRef>,
        validate_fn: None,
    }
}

pub(crate) fn make_extra_ref() -> FileType {
    FileType {
        key: "extra_ref",
        name: "Ext*.ref",
        description: "Special object placements",
        extensions: &[".ref"],
        detect_kind: DetectKind::RefPrefix("Ext"),
        extract_fn: extract_as::<crate::references::extra_ref::ExtraRef>,
        patch_fn: patch_as::<crate::references::extra_ref::ExtraRef>,
        validate_fn: Some(validate_as::<crate::references::extra_ref::ExtraRef>),
    }
}

pub(crate) fn make_event_npc_ref() -> FileType {
    FileType {
        key: "event_npc_ref",
        name: "Eventnpc.ref",
        description: "Event NPC placements",
        extensions: &[".ref"],
        detect_kind: DetectKind::RefPrefix("Eventnpc"),
        extract_fn: extract_as::<crate::references::event_npc_ref::EventNpcRef>,
        patch_fn: patch_as::<crate::references::event_npc_ref::EventNpcRef>,
        validate_fn: None,
    }
}

pub(crate) fn make_dialog() -> FileType {
    FileType {
        key: "dialog",
        name: "*.dlg",
        description: "Dialogue script CSV",
        extensions: &[".dlg"],
        detect_kind: DetectKind::DlgPrefix("Dlg"),
        extract_fn: extract_as::<crate::references::dialog::Dialog>,
        patch_fn: patch_as::<crate::references::dialog::Dialog>,
        validate_fn: Some(validate_as::<crate::references::dialog::Dialog>),
    }
}

pub(crate) fn make_dialog_text() -> FileType {
    FileType {
        key: "dialog_text",
        name: "*.pgp",
        description: "Dialogue text package",
        extensions: &[".pgp"],
        detect_kind: DetectKind::PgpPrefix("Pgp"),
        extract_fn: extract_as::<crate::references::dialogue_text::DialogueText>,
        patch_fn: patch_as::<crate::references::dialogue_text::DialogueText>,
        validate_fn: Some(validate_as::<crate::references::dialogue_text::DialogueText>),
    }
}

pub(crate) fn make_quest() -> FileType {
    FileType {
        key: "quest",
        name: "Quest.scr",
        description: "Quest definitions",
        extensions: &[".scr"],
        detect_kind: DetectKind::Scr("Quest.scr"),
        extract_fn: extract_as::<crate::references::quest_scr::Quest>,
        patch_fn: patch_as::<crate::references::quest_scr::Quest>,
        validate_fn: None,
    }
}

pub(crate) fn make_message() -> FileType {
    FileType {
        key: "message",
        name: "Message.scr",
        description: "Diary game messages",
        extensions: &[".scr"],
        detect_kind: DetectKind::Scr("Message.scr"),
        extract_fn: extract_as::<crate::references::message_scr::Message>,
        patch_fn: patch_as::<crate::references::message_scr::Message>,
        validate_fn: None,
    }
}

pub(crate) fn make_map_file() -> FileType {
    FileType {
        key: "map_file",
        name: "*.map",
        description: "Map geometry, sprites, events, tiles (extract only)",
        extensions: &[".map"],
        detect_kind: DetectKind::Db(&[]),
        extract_fn: extract_map_file,
        patch_fn: patch_not_supported,
        validate_fn: None,
    }
}

pub(crate) fn make_gtl() -> FileType {
    FileType {
        key: "gtl",
        name: "*.gtl",
        description: "Ground tile layer (extract only)",
        extensions: &[".gtl"],
        detect_kind: DetectKind::Db(&[]),
        extract_fn: extract_tileset,
        patch_fn: patch_not_supported,
        validate_fn: None,
    }
}

pub(crate) fn make_btl() -> FileType {
    FileType {
        key: "btl",
        name: "*.btl",
        description: "Building tile layer (extract only)",
        extensions: &[".btl"],
        detect_kind: DetectKind::Db(&[]),
        extract_fn: extract_tileset,
        patch_fn: patch_not_supported,
        validate_fn: None,
    }
}

pub(crate) fn make_sprite() -> FileType {
    FileType {
        key: "sprite",
        name: "*.spr",
        description: "Sprite/animation file (extract only)",
        extensions: &[".spr"],
        detect_kind: DetectKind::Db(&[]),
        extract_fn: extract_sprite_info,
        patch_fn: patch_not_supported,
        validate_fn: None,
    }
}
