//! Tests that every `EditableRecord` implementation is consistent with its
//! corresponding struct definition in `dispel-extractor/src/references/`.
//!
//! Two invariants are enforced per type:
//!
//! 1. **No stale descriptor names** — every name in `field_descriptors()` must
//!    exist as an actual struct field (or be listed in `virtual_desc` if it is
//!    intentionally computed rather than stored).  A failure here means a struct
//!    field was renamed without updating the editor, which silently breaks
//!    `get_field` / `set_field`.
//!
//! 2. **Full struct coverage** — every struct field must appear in
//!    `field_descriptors()` or be listed in `skip_fields`.  A failure here
//!    means newly added (or previously overlooked) struct fields have no editor
//!    UI.  Add them to the editor or to `skip_fields` with a brief comment.
//!
//! Run with `cargo test field_coverage` to execute only these tests.

use std::collections::HashSet;

use crate::components::editor::editable::EditableRecord;

// ── AST helper ───────────────────────────────────────────────────────────────

/// Parse `source` as a Rust file and return the named fields of `struct_name`.
fn parse_struct_fields(source: &str, struct_name: &str) -> Vec<String> {
    let ast = syn::parse_file(source).unwrap_or_else(|e| panic!("syn parse error: {e}"));
    for item in ast.items {
        if let syn::Item::Struct(s) = item {
            if s.ident == struct_name {
                if let syn::Fields::Named(fields) = s.fields {
                    return fields
                        .named
                        .iter()
                        .filter_map(|f| f.ident.as_ref())
                        .map(|i| i.to_string())
                        .collect();
                }
            }
        }
    }
    panic!("struct `{struct_name}` not found in source");
}

// ── Core check macro ─────────────────────────────────────────────────────────

/// Generate a `#[test]` function that verifies field coverage for one type.
///
/// Parameters:
/// - `fn <name>` — test function name
/// - `type <T>` — the `EditableRecord` implementor
/// - `src "<path>"` — source file path relative to workspace root
/// - `struct "<Name>"` — struct name as it appears in the source
/// - `virtual_desc [...]` — descriptor names that are *computed*, not struct fields
/// - `skip_fields [...]` — struct field names intentionally absent from descriptors
macro_rules! check_record {
    (
        fn $test_name:ident,
        type $T:ty,
        src $rel_path:literal,
        struct $struct_name:literal,
        virtual_desc [$($virt:literal),*],
        skip_fields [$($skip:literal),*]
    ) => {
        #[test]
        fn $test_name() {
            // CARGO_MANIFEST_DIR points to dispel-gui/; go up one level to workspace root.
            let root = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
            let path = format!("{root}/{}", $rel_path);
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("cannot read {path}: {e}"));

            let struct_fields: HashSet<String> =
                parse_struct_fields(&source, $struct_name).into_iter().collect();

            let desc_names: HashSet<&str> = <$T as EditableRecord>::field_descriptors()
                .iter()
                .map(|d| d.name)
                .collect();

            // Computed descriptor names that have no backing struct field.
            let virtual_desc: HashSet<&str> = vec![$($virt),*].into_iter().collect();
            // Struct fields that are intentionally absent from the editor.
            let skip_fields: HashSet<&str> = vec![$($skip),*].into_iter().collect();

            // ── invariant 1: no stale descriptor names ────────────────────────
            let mut stale: Vec<&&str> = desc_names
                .iter()
                .filter(|&&n| !struct_fields.contains(n) && !virtual_desc.contains(n))
                .collect();
            stale.sort();
            assert!(
                stale.is_empty(),
                "{}: descriptor names not found in struct \
                 (field renamed without updating the editor?): {stale:?}",
                $struct_name
            );

            // ── invariant 2: full struct coverage ────────────────────────────
            let mut uncovered: Vec<&String> = struct_fields
                .iter()
                .filter(|f| !desc_names.contains(f.as_str()) && !skip_fields.contains(f.as_str()))
                .collect();
            uncovered.sort();
            assert!(
                uncovered.is_empty(),
                "{}: struct fields not covered by field_descriptors — \
                 either add them to the editor or to `skip_fields` with a comment: {uncovered:?}",
                $struct_name
            );
        }
    };
}

// ── Per-type tests ────────────────────────────────────────────────────────────

use dispel_core::{
    ChData, DialogueParagraph, DialogueScript, DrawItem, EditItem, Event, EventItem, EventNpcRef,
    Extra, ExtraRef, HealItem, MagicSpell, Map, MapIni, MiscItem, Monster, MonsterRef, NpcIni,
    PartyIniNpc, PartyLevelNpc, PartyRef, Quest, Store, WaveIni, WeaponItem, NPC,
};
// `Message` is the ScrMessage type in dispel_core.
use dispel_core::Message as ScrMessage;

check_record!(
    fn all_map_ini_fields_covered,
    type Map,
    src "src/references/all_map_ini.rs",
    struct "Map",
    virtual_desc [],
    skip_fields []
);

check_record!(
    fn chdata_fields_covered,
    type ChData,
    src "src/references/chdata_db.rs",
    struct "ChData",
    virtual_desc [],
    skip_fields []
);

check_record!(
    fn dialog_fields_covered,
    type DialogueScript,
    src "src/references/dialogue_script.rs",
    struct "DialogueScript",
    virtual_desc [],
    skip_fields [
        "next_dialog_id1",  // choice dialog option 1
        "next_dialog_id2",  // choice dialog option 2
        "next_dialog_id3"   // choice dialog option 3
    ]
);

check_record!(
    fn dialogue_paragraph_fields_covered,
    type DialogueParagraph,
    src "src/references/dialogue_paragraph.rs",
    struct "DialogueParagraph",
    virtual_desc [],
    skip_fields []
);

check_record!(
    fn draw_item_fields_covered,
    type DrawItem,
    src "src/references/draw_item.rs",
    struct "DrawItem",
    virtual_desc [],
    skip_fields []
);

check_record!(
    fn edit_item_fields_covered,
    type EditItem,
    src "src/references/edit_item_db.rs",
    struct "EditItem",
    virtual_desc [],
    skip_fields [
        "index",    // auto-incremented position in file, not user-editable
        "padding1", // binary padding, no semantic meaning
        "padding2", // binary padding, no semantic meaning
        "padding3", // binary padding, no semantic meaning
        "padding4"  // binary padding, no semantic meaning
    ]
);

check_record!(
    fn event_ini_fields_covered,
    type Event,
    src "src/references/event_ini.rs",
    struct "Event",
    virtual_desc [],
    skip_fields []
);

check_record!(
    fn event_item_fields_covered,
    type EventItem,
    src "src/references/event_item_db.rs",
    struct "EventItem",
    virtual_desc [],
    skip_fields [
        "id",      // positional index, not user-editable
        "padding"  // binary padding, no semantic meaning
    ]
);

check_record!(
    fn event_npc_ref_fields_covered,
    type EventNpcRef,
    src "src/references/event_npc_ref.rs",
    struct "EventNpcRef",
    virtual_desc [],
    skip_fields []
);

check_record!(
    fn extra_ini_fields_covered,
    type Extra,
    src "src/references/extra_ini.rs",
    struct "Extra",
    virtual_desc [],
    skip_fields []
);

check_record!(
    fn extra_ref_fields_covered,
    type ExtraRef,
    src "src/references/extra_ref.rs",
    struct "ExtraRef",
    virtual_desc [],
    skip_fields [
        // Positional / internal metadata
        "number_in_file",
        // Not yet reverse-engineered — unknown byte sequences
        "unknown1",
        "unknown2",
        "unknown3",
        "unknown4",
        "unknown5",
        "unknown6",
        "unknown7",
        "unknown8",
        "unknown9",
        "unknown10",
        "unknown11",
        "unknown12",
        "unknown13",
        "unknown14",
        "unknown15",
        "unknown16",
        "unknown17",
        "unknown18",
        "unknown20",
        "unknown21",
        "unknown22",
        "unknown23",
        "unknown24",
        "unknown25",
        "unknown26",
        "unknown27",
        // Second required-item slot not yet exposed in the editor
        "required_item_id2",
        "required_item_type_id2"
    ]
);

check_record!(
    fn heal_item_fields_covered,
    type HealItem,
    src "src/references/heal_item_db.rs",
    struct "HealItem",
    virtual_desc [],
    skip_fields [
        "id",       // positional index
        "padding1", // binary padding
        "padding2",
        "padding3",
        "padding4",
        "padding5"
    ]
);

check_record!(
    fn magic_spell_fields_covered,
    type MagicSpell,
    src "src/references/magic_db.rs",
    struct "MagicSpell",
    virtual_desc [],
    skip_fields [
        "id",        // positional index
        "flag1",     // internal engine flag
        "flag2",
        "flag3",
        "reserved1", // reserved/padding bytes
        "reserved2",
        "reserved3",
        "reserved4",
        "constant1"  // engine constant, not modder-relevant
    ]
);

check_record!(
    fn map_ini_fields_covered,
    type MapIni,
    src "src/references/map_ini.rs",
    struct "MapIni",
    virtual_desc [],
    skip_fields []
);

check_record!(
    fn message_scr_fields_covered,
    type ScrMessage,
    src "src/references/message_scr.rs",
    struct "Message",
    virtual_desc [],
    skip_fields []
);

check_record!(
    fn misc_item_fields_covered,
    type MiscItem,
    src "src/references/misc_item_db.rs",
    struct "MiscItem",
    virtual_desc [],
    skip_fields [
        "id",      // positional index
        "padding"  // binary padding, no semantic meaning
    ]
);

check_record!(
    fn monster_db_fields_covered,
    type Monster,
    src "src/references/monster_db.rs",
    struct "Monster",
    virtual_desc [],
    skip_fields []
);

check_record!(
    fn monster_ref_fields_covered,
    type MonsterRef,
    src "src/references/monster_ref.rs",
    struct "MonsterRef",
    virtual_desc [],
    skip_fields [
        "index" // auto-incremented position in file, not user-editable
    ]
);

check_record!(
    fn npc_ini_fields_covered,
    type NpcIni,
    src "src/references/npc_ini.rs",
    struct "NpcIni",
    virtual_desc [],
    skip_fields []
);

check_record!(
    fn npc_ref_fields_covered,
    type NPC,
    src "src/references/npc_ref.rs",
    struct "NPC",
    virtual_desc [],
    skip_fields [
        "index",        // auto-incremented position in file
        "description",  // NPC description/role
        "unknown_1",    // unknown field
        "goto1_filled", // internal "is waypoint active" flags derived from coords
        "goto2_filled",
        "goto3_filled",
        "goto4_filled",
        "unknown_2",    // unknown field
        "unknown_3",    // unknown field
        "unknown_4",    // unknown field
        "unknown_5",    // unknown field
        "unknown_6",    // unknown field
        "unknown_7",    // unknown field
        "unknown_8",    // unknown field
        "unknown_10",   // unknown field
        "unknown_11",   // unknown field
        "unknown_12",   // unknown field
        "unknown_13",   // unknown field
        "unknown_14",   // unknown field
        "unknown_15",   // unknown field
        "unknown_16",   // unknown field
        "unknown_20",    // unknown field
        "unknown_9",     // unknown field
        "unknown_17",    // unknown field
        "unknown_18",    // unknown field
        "unknown_19"     // unknown field
    ]
);

check_record!(
    fn party_ini_db_fields_covered,
    type PartyIniNpc,
    src "src/references/party_ini_db.rs",
    struct "PartyIniNpc",
    virtual_desc [],
    skip_fields []
);

check_record!(
    fn party_level_db_fields_covered,
    type PartyLevelNpc,
    src "src/references/party_level_db.rs",
    struct "PartyLevelNpc",
    virtual_desc [
        // Computed display-only field: returns self.records.len()
        "records_count"
    ],
    skip_fields [
        // Level-progression stats live in PartyLevelRecord children,
        // not directly on PartyLevelNpc — editing them requires a nested editor.
        "level",
        "strength",
        "constitution",
        "wisdom",
        "health_points",
        "mana_points",
        "agility",
        "attack",
        "mana_recharge",
        "defense",
        "records" // Vec<PartyLevelRecord> — nested, handled separately
    ]
);

check_record!(
    fn party_ref_fields_covered,
    type PartyRef,
    src "src/references/party_ref.rs",
    struct "PartyRef",
    virtual_desc [],
    skip_fields []
);

check_record!(
    fn quest_scr_fields_covered,
    type Quest,
    src "src/references/quest_scr.rs",
    struct "Quest",
    virtual_desc [],
    skip_fields []
);

check_record!(
    fn store_db_fields_covered,
    type Store,
    src "src/references/store_db.rs",
    struct "Store",
    virtual_desc [],
    skip_fields [
        // Vec<StoreProduct> — nested complex type, requires its own editor
        "products"
    ]
);

check_record!(
    fn wave_ini_fields_covered,
    type WaveIni,
    src "src/references/wave_ini.rs",
    struct "WaveIni",
    virtual_desc [],
    skip_fields []
);

check_record!(
    fn weapon_item_fields_covered,
    type WeaponItem,
    src "src/references/weapons_db.rs",
    struct "WeaponItem",
    virtual_desc [],
    skip_fields [
        "id" // positional index
    ]
);
