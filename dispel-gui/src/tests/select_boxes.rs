//! Roundtrip audit of every select box the GUI renders.
//!
//! Enum fields are enumerated from the real wire values of their record enums;
//! lookup and composite fields are exercised with invented catalogs that both
//! contain and omit the displayed id. Each label must survive `set_field` /
//! `get_field` and be visible in the spreadsheet inspector and the map editor.

use std::collections::HashMap;
use std::fmt::{Debug, Display};

use dispel_core::references::enums::WireValue;
use dispel_core::{
    BooleanFlag, DialogOwner, DialogType, DialogueScript, EditItem, EditItemEffect,
    EditItemModification, Event, EventNpcRef, EventType, ExtraObjectType, ExtraRef, HealItem,
    HealItemFlag, MagicSpell, MagicSpellFlag, MagicType, Map, MapLighting, Monster, MonsterAiType,
    MonsterRef, MonsterSpawn, NPC, NpcInteractionMode, NpcLookingDirection, NpcMovementMode,
    NpcRoleResult, SmallRange0to3, SpellTargetType,
};

use crate::components::editable::{EditableRecord, FieldDescriptor, FieldKind};
use crate::editors::map_editor::message::SelectedEntity;
use crate::editors::map_editor::view::fields;
use crate::message::{EditorMessage, Message, SystemMessage};
use crate::view::editor::spreadsheet::message::SpreadsheetMessage;
use crate::view::editor::spreadsheet::view::inspector::build_inspector_field;

use super::common::{activate_select_option, select_shows};

/// The formatting protocol of a field's get path.
trait LabelFormat<E> {
    fn label(value: &E) -> String;
}

struct DebugFormat;
struct DisplayFormat;

impl<E: Debug> LabelFormat<E> for DebugFormat {
    fn label(value: &E) -> String {
        format!("{value:?}")
    }
}

impl<E: Display> LabelFormat<E> for DisplayFormat {
    fn label(value: &E) -> String {
        value.to_string()
    }
}

/// Wire widths used by the enum fields under audit.
trait WireScan: Copy {
    fn scan() -> Vec<Self>;
}

impl WireScan for i32 {
    fn scan() -> Vec<Self> {
        (-1..=512).collect()
    }
}

impl WireScan for i16 {
    fn scan() -> Vec<Self> {
        (-1..=512).collect()
    }
}

impl WireScan for u8 {
    fn scan() -> Vec<Self> {
        (0..=255).collect()
    }
}

impl WireScan for u32 {
    fn scan() -> Vec<Self> {
        (0..=512).collect()
    }
}

fn wire_labels<E, W, Fmt>(_format: Fmt) -> Vec<String>
where
    E: WireValue<W> + Default,
    W: WireScan,
    Fmt: LabelFormat<E>,
{
    let mut labels: Vec<String> = W::scan()
        .into_iter()
        .filter_map(|wire| <E as WireValue<W>>::from_wire_checked(wire))
        .map(|value| Fmt::label(&value))
        .collect();
    labels.sort();
    labels.dedup();
    labels
}

fn descriptor_of<R: EditableRecord>(name: &'static str) -> &'static FieldDescriptor {
    R::field_descriptors()
        .iter()
        .find(|desc| desc.name == name)
        .unwrap_or_else(|| panic!("missing descriptor for {name}"))
}

fn captured_field_changed(_: usize, _: String, _: String) -> Message {
    Message::System(SystemMessage::DismissError)
}

fn captured_spreadsheet(msg: SpreadsheetMessage) -> Message {
    match msg {
        SpreadsheetMessage::InspectorFieldChanged(idx, name, value) => {
            Message::Editor(EditorMessage::Weapon(
                crate::editors::weapon::WeaponEditorMessage::FieldChanged(idx, name, value),
            ))
        }
        _ => Message::System(SystemMessage::DismissError),
    }
}

fn render_both(
    desc: &FieldDescriptor,
    value: &str,
    lookups: &HashMap<String, Vec<(String, String)>>,
    expected: &str,
) {
    let empty = HashMap::new();
    let inspector = build_inspector_field(
        desc,
        value.to_string(),
        0,
        lookups,
        None,
        captured_field_changed,
        &empty,
        captured_spreadsheet,
    );
    assert!(
        select_shows(inspector, expected),
        "inspector for {} must render {expected:?}",
        desc.name
    );

    let row = fields::inspector_field_row(
        desc.label,
        desc.name,
        &desc.kind,
        value,
        0,
        SelectedEntity::Npc(0),
        lookups,
    );
    assert!(
        select_shows(row, expected),
        "map editor row for {} must render {expected:?}",
        desc.name
    );
}

fn audit_enum<R, E, W, Fmt>(field: &'static str, format: Fmt)
where
    R: EditableRecord,
    E: WireValue<W> + Default,
    W: WireScan,
    Fmt: LabelFormat<E>,
{
    let desc = descriptor_of::<R>(field);
    let FieldKind::Enum { variants } = &desc.kind else {
        panic!("{field} is not an enum field");
    };
    let lookups = HashMap::new();

    for label in wire_labels::<E, W, Fmt>(format) {
        assert!(
            variants.contains(&label.as_str()),
            "{field}: wire label {label:?} is missing from {:?}",
            variants
        );
        let mut record = R::default();
        assert!(
            record.set_field(field, label.clone()),
            "{field}: cannot set {label:?}"
        );
        assert_eq!(
            record.get_field(field),
            label,
            "{field}: get_field disagrees with the rendered label"
        );
        render_both(desc, &label, &lookups, &label);
    }
}

fn inspector_emissions(
    desc: &FieldDescriptor,
    value: &str,
    lookups: &HashMap<String, Vec<(String, String)>>,
    nth: usize,
) -> Vec<String> {
    let empty = HashMap::new();
    let element = build_inspector_field(
        desc,
        value.to_string(),
        0,
        lookups,
        None,
        captured_field_changed,
        &empty,
        captured_spreadsheet,
    );
    activate_select_option(element, nth, 0)
        .into_iter()
        .filter_map(|message| match message {
            Message::Editor(EditorMessage::Weapon(
                crate::editors::weapon::WeaponEditorMessage::FieldChanged(_, _, value),
            )) => Some(value),
            _ => None,
        })
        .collect()
}

fn map_emissions(
    desc: &FieldDescriptor,
    value: &str,
    lookups: &HashMap<String, Vec<(String, String)>>,
) -> Vec<String> {
    let element = fields::inspector_field_row(
        desc.label,
        desc.name,
        &desc.kind,
        value,
        0,
        SelectedEntity::Npc(0),
        lookups,
    );
    activate_select_option(element, 0, 0)
        .into_iter()
        .filter_map(|message| match message {
            Message::Editor(EditorMessage::MapEditor(
                crate::editors::map_editor::MapEditorMessage::EntityFieldChanged(_, _, _, value),
            )) => Some(value),
            _ => None,
        })
        .collect()
}

fn audit_lookup<R: EditableRecord>(field: &'static str) {
    let desc = descriptor_of::<R>(field);
    let FieldKind::Lookup(key) = &desc.kind else {
        panic!("{field} is not a lookup field");
    };
    let entries = vec![
        ("7".to_string(), "Invented 7".to_string()),
        ("9".to_string(), "Invented 9".to_string()),
    ];
    let present = HashMap::from([(key.to_string(), entries.clone())]);
    let absent = HashMap::from([(
        key.to_string(),
        vec![("7".to_string(), "Invented 7".to_string())],
    )]);

    render_both(desc, "9", &present, "Invented 9");
    render_both(desc, "9", &absent, "Unknown (9)");

    for emitted in inspector_emissions(desc, "9", &present, 0)
        .into_iter()
        .chain(map_emissions(desc, "9", &present))
    {
        let mut record = R::default();
        assert!(
            record.set_field(field, emitted.clone()),
            "{field}: emitted id {emitted:?} was rejected"
        );
        let stored = record.get_field(field);
        assert_eq!(stored, emitted, "{field}: emitted id did not round-trip");
        render_both(desc, &stored, &present, &format!("Invented {stored}"));
    }
}

fn audit_composite<R: EditableRecord>(field: &'static str) {
    let desc = descriptor_of::<R>(field);
    let FieldKind::CompositeItem { lookup_key } = &desc.kind else {
        panic!("{field} is not a composite field");
    };
    let present = HashMap::from([(
        lookup_key.to_string(),
        vec![
            ("1:5".to_string(), "Invented Sword".to_string()),
            ("255:15".to_string(), "[-]".to_string()),
        ],
    )]);
    let absent = HashMap::from([(
        lookup_key.to_string(),
        vec![
            ("1:6".to_string(), "Invented Shield".to_string()),
            ("255:15".to_string(), "[-]".to_string()),
        ],
    )]);

    render_both(desc, "1:5", &present, "Invented Sword");
    render_both(desc, "1:5", &absent, "Unknown (5)");
    render_both(desc, "1:99", &present, "Unknown (99)");

    for emitted in inspector_emissions(desc, "1:5", &present, 1)
        .into_iter()
        .chain(map_emissions(desc, "1:5", &present))
    {
        let mut record = R::default();
        assert!(
            record.set_field(field, emitted.clone()),
            "{field}: emitted key {emitted:?} was rejected"
        );
        let stored = record.get_field(field);
        assert_eq!(stored, emitted, "{field}: emitted key did not round-trip");
        render_both(desc, &stored, &present, "Invented Sword");
    }
}

#[test]
fn test_every_enum_field_round_trips_through_both_views() {
    audit_enum::<Monster, MonsterAiType, i32, _>("ai_type", DisplayFormat);
    audit_enum::<MonsterSpawn, BooleanFlag, i32, _>("persistent", DisplayFormat);
    audit_enum::<MonsterSpawn, BooleanFlag, i32, _>("burrow", DisplayFormat);
    audit_enum::<DialogueScript, DialogType, i32, _>("dialog_type", DisplayFormat);
    audit_enum::<DialogueScript, DialogOwner, i32, _>("dialog_owner", DisplayFormat);
    audit_enum::<ExtraRef, ExtraObjectType, u8, _>("object_type", DebugFormat);
    audit_enum::<ExtraRef, BooleanFlag, i32, _>("requires_key", DisplayFormat);
    audit_enum::<ExtraRef, SmallRange0to3, i32, _>("footprint_width", DisplayFormat);
    audit_enum::<ExtraRef, SmallRange0to3, i32, _>("footprint_height", DisplayFormat);
    for field in [
        "is_quest_element",
        "post_activation_tile_flag",
        "post_activation_footprint_mode",
        "alternate_render_mode",
        "active_overlay_enabled",
        "map_object_active",
    ] {
        audit_enum::<ExtraRef, BooleanFlag, i32, _>(field, DisplayFormat);
    }
    audit_enum::<Map, MapLighting, i32, _>("lighting", DisplayFormat);
    audit_enum::<EventNpcRef, NpcRoleResult, i32, _>("role_result", DebugFormat);
    audit_enum::<EventNpcRef, NpcMovementMode, i32, _>("movement_mode", DebugFormat);
    audit_enum::<EventNpcRef, NpcInteractionMode, i32, _>("interaction_mode", DebugFormat);
    for field in [
        "waypoint1_filled",
        "waypoint2_filled",
        "waypoint3_filled",
        "waypoint4_filled",
    ] {
        audit_enum::<EventNpcRef, BooleanFlag, i32, _>(field, DebugFormat);
    }
    for field in [
        "waypoint1_facing_direction",
        "waypoint2_facing_direction",
        "waypoint3_facing_direction",
        "waypoint4_facing_direction",
    ] {
        audit_enum::<EventNpcRef, NpcLookingDirection, i32, _>(field, DebugFormat);
    }
    audit_enum::<Event, EventType, i32, _>("event_type", DisplayFormat);
    for field in [
        "initial_patrol_countdown",
        "skip_ai_action",
        "force_ai_update",
    ] {
        audit_enum::<MonsterRef, BooleanFlag, i32, _>(field, DisplayFormat);
    }
    for field in [
        "restores_full_health",
        "restores_full_mana",
        "cures_poison",
        "cures_petrification",
        "cures_polymorph",
    ] {
        audit_enum::<HealItem, HealItemFlag, u8, _>(field, DebugFormat);
    }
    audit_enum::<NPC, NpcRoleResult, i32, _>("role_result", DisplayFormat);
    audit_enum::<NPC, NpcMovementMode, i32, _>("movement_mode", DisplayFormat);
    audit_enum::<NPC, NpcInteractionMode, i32, _>("interaction_mode", DisplayFormat);
    audit_enum::<NPC, BooleanFlag, i32, _>("goto1_filled", DebugFormat);
    audit_enum::<NPC, BooleanFlag, i32, _>("goto2_filled", DebugFormat);
    audit_enum::<NPC, BooleanFlag, i32, _>("goto3_filled", DebugFormat);
    audit_enum::<NPC, BooleanFlag, i32, _>("goto4_filled", DebugFormat);
    for field in [
        "waypoint1_facing_direction",
        "waypoint2_facing_direction",
        "waypoint3_facing_direction",
        "waypoint4_facing_direction",
    ] {
        audit_enum::<NPC, NpcLookingDirection, i32, _>(field, DebugFormat);
    }
    audit_enum::<EditItem, EditItemModification, u8, _>("modifies_item", DebugFormat);
    audit_enum::<EditItem, EditItemEffect, i16, _>("additional_effect", DebugFormat);
    audit_enum::<MagicSpell, MagicType, u32, _>("magic_type", DebugFormat);
    audit_enum::<MagicSpell, SpellTargetType, u32, _>("targeting_mode", DebugFormat);
    audit_enum::<MagicSpell, MagicSpellFlag, u32, _>("enabled", DebugFormat);
}

#[test]
fn test_store_product_modal_labels_known_and_unknown_types() {
    use crate::editors::store::StoreEditorState;
    use crate::editors::store::view::product_modal;

    let known = StoreEditorState {
        modal_edit_type: 2,
        ..Default::default()
    };
    assert!(
        super::common::select_values(product_modal(&known, &None, &None, &None, &None))
            .iter()
            .any(|value| value == "HealItem")
    );

    let unknown = StoreEditorState {
        modal_edit_type: 9,
        ..Default::default()
    };
    let values = super::common::select_values(product_modal(&unknown, &None, &None, &None, &None));
    assert!(
        values.iter().any(|value| value.contains('9')),
        "an unknown product type must stay visible, got {values:?}"
    );
}

#[test]
fn test_localization_filter_only_survives_a_rescan_that_keeps_its_file() {
    use crate::editors::localization_manager::LocalizationMessage;
    use crate::editors::localization_manager::view::view;
    use crate::message::MessageExt;
    use dispel_core::TextEntry;
    use dispel_core::localization::TextEncoding;

    fn entry(file: &str) -> TextEntry {
        TextEntry {
            file_path: file.to_string(),
            record_id: 0,
            field_name: "name",
            original: "Invented".to_string(),
            translation: String::new(),
            encoding: TextEncoding::Utf8,
            max_bytes: 64,
        }
    }

    let mut app = crate::app::App::test_new(crate::workspace::Workspace::new());
    app.state.editors.localization_manager.entries = vec![entry("Invented/Kept.ini")];
    app.state.editors.localization_manager.filter_file = Some("Invented/Kept.ini".to_string());
    assert!(select_shows(view(&app), "Kept.ini"));

    let _ = app.update(Message::localization(LocalizationMessage::Scanned(Ok(
        vec![entry("Invented/Other.ini")],
    ))));
    assert_eq!(app.state.editors.localization_manager.filter_file, None);
    assert!(select_shows(view(&app), "All files"));
}

#[test]
fn test_event_script_prefix_options_keep_the_current_prefix() {
    use dispel_core::references::event_scr::{ActionFunction, EventScript, SpriteDefinition};

    let mut script = EventScript::default();
    script.spr_content.push(SpriteDefinition {
        sprite_alias: "InventedAlias".to_string(),
        sprite_file: "Invented.spr".to_string(),
    });
    let current = ActionFunction {
        prefix: Some("InventedAlias".to_string()),
        function_name: "invented_call".to_string(),
        parameters: Vec::new(),
        raw_content: None,
    };
    let options = crate::editors::event_scr::view::build_prefix_options(&script, &current);
    assert!(options.iter().any(|value| value == "InventedAlias"));

    let stale = ActionFunction {
        prefix: Some("GhostPrefix".to_string()),
        function_name: "invented_call".to_string(),
        parameters: Vec::new(),
        raw_content: None,
    };
    let options = crate::editors::event_scr::view::build_prefix_options(&script, &stale);
    assert!(options.iter().any(|value| value == "GhostPrefix"));
}

#[test]
fn test_every_lookup_and_composite_field_round_trips_through_both_views() {
    audit_lookup::<MonsterRef>("monster_db_id");
    audit_composite::<dispel_core::DrawItem>("item");
    audit_composite::<ExtraRef>("required_item");
    audit_composite::<ExtraRef>("required_item2");
    audit_composite::<ExtraRef>("loot_item");
    audit_composite::<MonsterRef>("loot_item_1");
    audit_composite::<MonsterRef>("loot_item_2");
    audit_composite::<MonsterRef>("loot_item_3");
    audit_composite::<NPC>("interaction_result_item");
}
