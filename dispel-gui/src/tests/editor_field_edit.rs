//! Field-change integration tests for editor types not covered elsewhere.
//!
//! Coverage gap identified in the staging-branch review: 23 of 37 editor
//! types had zero `handle()` calls with `FieldChanged` in any test before
//! this file was created.  This file brings coverage to the majority of
//! remaining editors, following the pattern established in
//! `recording_tests.rs` (inject state → dispatch FieldChanged → assert
//! mutation).

use crate::app::App;
use crate::workspace::Workspace;

// ============================================================================
// Macro-generated editors  (define_standard_editor! → StandardEditor<T>)
// ============================================================================

mod standard_editors {
    use super::*;

    // ── MonsterEditor (record: Monster) ────────────────────────────────────

    #[test]
    fn monster_editor_field_change_updates_catalog() {
        use dispel_core::Monster;

        let mut app = App::test_new(Workspace::new());
        let record = Monster {
            name: "OldMonster".to_string(),
            ..Default::default()
        };
        app.state.editors.monster_editor.catalog = Some(vec![record]);
        app.state.editors.monster_editor.refresh();
        app.state.editors.monster_editor.select(0);

        let _task = crate::editors::monster::handle(
            crate::editors::monster::MonsterEditorMessage::FieldChanged(
                0,
                "name".to_string(),
                "NewMonster".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.monster_editor.filtered[0].1.name, "NewMonster",
            "Monster name updated via FieldChanged"
        );
    }

    #[test]
    fn monster_editor_field_change_oob_is_noop() {
        use dispel_core::Monster;

        let mut app = App::test_new(Workspace::new());
        app.state.editors.monster_editor.catalog = Some(vec![Monster::default()]);
        app.state.editors.monster_editor.refresh();
        app.state.editors.monster_editor.select(0);

        let _task = crate::editors::monster::handle(
            crate::editors::monster::MonsterEditorMessage::FieldChanged(
                999,
                "name".to_string(),
                "Changed".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.monster_editor.filtered[0].1.name, "",
            "OOB FieldChanged is a no-op"
        );
    }

    // ── MonsterIniEditor (record: MonsterIni) ──────────────────────────────

    #[test]
    fn monster_ini_editor_field_change_updates_catalog() {
        use dispel_core::MonsterIni;

        let mut app = App::test_new(Workspace::new());
        let record = MonsterIni {
            name: Some("OldIni".to_string()),
            ..Default::default()
        };
        app.state.editors.monster_ini_editor.state.catalog = Some(vec![record]);
        app.state.editors.monster_ini_editor.state.refresh();
        app.state.editors.monster_ini_editor.state.select(0);

        let _task = crate::editors::monster_ini::handle(
            crate::editors::monster_ini::MonsterIniEditorMessage::FieldChanged(
                0,
                "name".to_string(),
                "NewIni".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.monster_ini_editor.state.filtered[0]
                .1
                .name,
            Some("NewIni".to_string()),
            "MonsterIni name updated"
        );
    }

    // ── MagicEditor (record: MagicSpell) ────────────────────────────────────

    #[test]
    fn magic_editor_field_change_updates_catalog() {
        use dispel_core::MagicSpell;

        let mut app = App::test_new(Workspace::new());
        let record = MagicSpell {
            mana_cost: 10,
            ..Default::default()
        };
        app.state.editors.magic_editor.state.catalog = Some(vec![record]);
        app.state.editors.magic_editor.state.refresh();
        app.state.editors.magic_editor.state.select(0);

        let _task = crate::editors::magic::handle(
            crate::editors::magic::MagicEditorMessage::FieldChanged(
                0,
                "mana_cost".to_string(),
                "99".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.magic_editor.state.filtered[0].1.mana_cost, 99,
            "MagicSpell mana_cost updated"
        );
    }

    // ── QuestScrEditor (record: Quest) ──────────────────────────────────────

    #[test]
    fn quest_scr_editor_field_change_updates_catalog() {
        use dispel_core::Quest;

        let mut app = App::test_new(Workspace::new());
        let record = Quest {
            title: "Old Quest".to_string(),
            ..Default::default()
        };
        app.state.editors.quest_scr_editor.state.catalog = Some(vec![record]);
        app.state.editors.quest_scr_editor.state.refresh();
        app.state.editors.quest_scr_editor.state.select(0);

        let _task = crate::editors::quest_scr::handle(
            crate::editors::quest_scr::QuestScrEditorMessage::FieldChanged(
                0,
                "title".to_string(),
                "New Quest".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.quest_scr_editor.state.filtered[0].1.title, "New Quest",
            "Quest title updated"
        );
    }

    #[test]
    fn quest_scr_editor_description_plain_string() {
        // Regression: QuestScr changed from Option<String> to String
        // so TextArea can bind directly.
        use dispel_core::Quest;

        let mut app = App::test_new(Workspace::new());
        let record = Quest {
            description: "".to_string(),
            ..Default::default()
        };
        app.state.editors.quest_scr_editor.state.catalog = Some(vec![record]);
        app.state.editors.quest_scr_editor.state.refresh();
        app.state.editors.quest_scr_editor.state.select(0);

        let _task = crate::editors::quest_scr::handle(
            crate::editors::quest_scr::QuestScrEditorMessage::FieldChanged(
                0,
                "description".to_string(),
                "Long description".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.quest_scr_editor.state.filtered[0]
                .1
                .description,
            "Long description",
            "Description (plain String) updated"
        );
    }

    // ── HealItemEditor (record: HealItem) ───────────────────────────────────

    #[test]
    fn heal_item_editor_field_change_updates_catalog() {
        use dispel_core::HealItem;

        let mut app = App::test_new(Workspace::new());
        let record = HealItem {
            name: "OldHeal".to_string(),
            ..Default::default()
        };
        app.state.editors.heal_item_editor.catalog = Some(vec![record]);
        app.state.editors.heal_item_editor.refresh();
        app.state.editors.heal_item_editor.select(0);

        let _task = crate::editors::heal_item::handle(
            crate::editors::heal_item::HealItemEditorMessage::FieldChanged(
                0,
                "name".to_string(),
                "NewHeal".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.heal_item_editor.filtered[0].1.name, "NewHeal",
            "HealItem name updated"
        );
    }

    // ── MiscItemEditor (record: MiscItem) ───────────────────────────────────

    #[test]
    fn misc_item_editor_field_change_updates_catalog() {
        use dispel_core::MiscItem;

        let mut app = App::test_new(Workspace::new());
        let record = MiscItem {
            name: "OldMisc".to_string(),
            ..Default::default()
        };
        app.state.editors.misc_item_editor.catalog = Some(vec![record]);
        app.state.editors.misc_item_editor.refresh();
        app.state.editors.misc_item_editor.select(0);

        let _task = crate::editors::misc_item::handle(
            crate::editors::misc_item::MiscItemEditorMessage::FieldChanged(
                0,
                "name".to_string(),
                "NewMisc".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.misc_item_editor.filtered[0].1.name, "NewMisc",
            "MiscItem name updated"
        );
    }

    // ── EventItemEditor (record: EventItem) ─────────────────────────────────

    #[test]
    fn event_item_editor_field_change_updates_catalog() {
        use dispel_core::EventItem;

        let mut app = App::test_new(Workspace::new());
        let record = EventItem {
            name: "OldEvent".to_string(),
            ..Default::default()
        };
        app.state.editors.event_item_editor.catalog = Some(vec![record]);
        app.state.editors.event_item_editor.refresh();
        app.state.editors.event_item_editor.select(0);

        let _task = crate::editors::event_item::handle(
            crate::editors::event_item::EventItemEditorMessage::FieldChanged(
                0,
                "name".to_string(),
                "NewEvent".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.event_item_editor.filtered[0].1.name, "NewEvent",
            "EventItem name updated"
        );
    }

    // ── EditItemEditor (record: EditItem) ────────────────────────────────────

    #[test]
    fn edit_item_editor_field_change_updates_catalog() {
        use dispel_core::EditItem;

        let mut app = App::test_new(Workspace::new());
        let record = EditItem {
            name: "OldEdit".to_string(),
            ..Default::default()
        };
        app.state.editors.edit_item_editor.catalog = Some(vec![record]);
        app.state.editors.edit_item_editor.refresh();
        app.state.editors.edit_item_editor.select(0);

        let _task = crate::editors::edit_item::handle(
            crate::editors::edit_item::EditItemEditorMessage::FieldChanged(
                0,
                "name".to_string(),
                "NewEdit".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.edit_item_editor.filtered[0].1.name, "NewEdit",
            "EditItem name updated"
        );
    }

    // ── EventIniEditor (record: Event) ──────────────────────────────────────

    #[test]
    fn event_ini_editor_field_change_updates_catalog() {
        use dispel_core::Event;

        let mut app = App::test_new(Workspace::new());
        let record = Event {
            event_id: 1,
            ..Default::default()
        };
        app.state.editors.event_ini_editor.state.catalog = Some(vec![record]);
        app.state.editors.event_ini_editor.state.refresh();
        app.state.editors.event_ini_editor.state.select(0);

        let _task = crate::editors::event_ini::handle(
            crate::editors::event_ini::EventIniEditorMessage::FieldChanged(
                0,
                "event_id".to_string(),
                "42".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.event_ini_editor.state.filtered[0]
                .1
                .event_id,
            42,
            "Event event_id updated"
        );
    }

    // ── ExtraIniEditor (record: Extra) ──────────────────────────────────────

    #[test]
    fn extra_ini_editor_field_change_updates_catalog() {
        use dispel_core::Extra;

        let mut app = App::test_new(Workspace::new());
        let record = Extra {
            id: 1,
            ..Default::default()
        };
        app.state.editors.extra_ini_editor.state.catalog = Some(vec![record]);
        app.state.editors.extra_ini_editor.state.refresh();
        app.state.editors.extra_ini_editor.state.select(0);

        let _task = crate::editors::extra_ini::handle(
            crate::editors::extra_ini::ExtraIniEditorMessage::FieldChanged(
                0,
                "id".to_string(),
                "99".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.extra_ini_editor.state.filtered[0].1.id, 99,
            "Extra id updated"
        );
    }

    // ── MapIniEditor (record: MapIni) ───────────────────────────────────────

    #[test]
    fn map_ini_editor_field_change_updates_catalog() {
        use dispel_core::MapIni;

        let mut app = App::test_new(Workspace::new());
        let record = MapIni {
            id: 1,
            ..Default::default()
        };
        app.state.editors.map_ini_editor.state.catalog = Some(vec![record]);
        app.state.editors.map_ini_editor.state.refresh();
        app.state.editors.map_ini_editor.state.select(0);

        let _task = crate::editors::map_ini::handle(
            crate::editors::map_ini::MapIniEditorMessage::FieldChanged(
                0,
                "id".to_string(),
                "42".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.map_ini_editor.state.filtered[0].1.id, 42,
            "MapIni id updated"
        );
    }

    // ── MessageScrEditor (record: Message) ──────────────────────────────────

    #[test]
    fn message_scr_editor_field_change_updates_catalog() {
        use dispel_core::Message;

        let mut app = App::test_new(Workspace::new());
        let record = Message {
            line1: Some("old text".to_string()),
            ..Default::default()
        };
        app.state.editors.message_scr_editor.state.catalog = Some(vec![record]);
        app.state.editors.message_scr_editor.state.refresh();
        app.state.editors.message_scr_editor.state.select(0);

        let _task = crate::editors::message_scr::handle(
            crate::editors::message_scr::MessageScrEditorMessage::FieldChanged(
                0,
                "line1".to_string(),
                "new text".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.message_scr_editor.state.filtered[0]
                .1
                .line1,
            Some("new text".to_string()),
            "Message line1 updated"
        );
    }

    // ── PartyRefEditor (record: PartyRef) ────────────────────────────────────

    #[test]
    fn party_ref_editor_field_change_updates_catalog() {
        use dispel_core::PartyRef;

        let mut app = App::test_new(Workspace::new());
        let record = PartyRef {
            npc_id: 1,
            ..Default::default()
        };
        app.state.editors.party_ref_editor.catalog = Some(vec![record]);
        app.state.editors.party_ref_editor.refresh();
        app.state.editors.party_ref_editor.select(0);

        let _task = crate::editors::party_ref::handle(
            crate::editors::party_ref::PartyRefEditorMessage::FieldChanged(
                0,
                "npc_id".to_string(),
                "42".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.party_ref_editor.filtered[0].1.npc_id, 42,
            "PartyRef npc_id updated"
        );
    }

    // ── PartyIniEditor (record: PartyIniNpc) ────────────────────────────────

    #[test]
    fn party_ini_editor_field_change_updates_catalog() {
        use dispel_core::PartyIniNpc;

        let mut app = App::test_new(Workspace::new());
        let record = PartyIniNpc {
            name: "OldName".to_string(),
            ..Default::default()
        };
        app.state.editors.party_ini_editor.state.catalog = Some(vec![record]);
        app.state.editors.party_ini_editor.state.refresh();
        app.state.editors.party_ini_editor.state.select(0);

        let _task = crate::editors::party_ini::handle(
            crate::editors::party_ini::PartyIniEditorMessage::FieldChanged(
                0,
                "name".to_string(),
                "NewName".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.party_ini_editor.state.filtered[0].1.name, "NewName",
            "PartyIniNpc name updated"
        );
    }

    // ── AllMapIniEditor (record: Map) ───────────────────────────────────────

    #[test]
    fn all_map_ini_editor_field_change_updates_catalog() {
        use dispel_core::Map;

        let mut app = App::test_new(Workspace::new());
        let record = Map {
            map_name: "Old Map".to_string(),
            ..Default::default()
        };
        app.state.editors.all_map_ini_editor.state.catalog = Some(vec![record]);
        app.state.editors.all_map_ini_editor.state.refresh();
        app.state.editors.all_map_ini_editor.state.select(0);

        let _task = crate::editors::all_map_ini::handle(
            crate::editors::all_map_ini::AllMapIniEditorMessage::FieldChanged(
                0,
                "map_name".to_string(),
                "New Map".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.all_map_ini_editor.state.filtered[0]
                .1
                .map_name,
            "New Map",
            "AllMapIni map_name updated"
        );
    }

    // ── NpcIniEditor (record: NpcIni) ───────────────────────────────────────

    #[test]
    fn npc_ini_editor_field_change_updates_catalog() {
        use dispel_core::NpcIni;

        let mut app = App::test_new(Workspace::new());
        let record = NpcIni {
            id: 1,
            ..Default::default()
        };
        app.state.editors.npc_ini_editor.state.catalog = Some(vec![record]);
        app.state.editors.npc_ini_editor.state.refresh();
        app.state.editors.npc_ini_editor.state.select(0);

        let _task = crate::editors::npc_ini::handle(
            crate::editors::npc_ini::NpcIniEditorMessage::FieldChanged(
                0,
                "id".to_string(),
                "42".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.npc_ini_editor.state.filtered[0].1.id, 42,
            "NpcIni id updated"
        );
    }

    // ── ChDataEditor (record: ChData) ───────────────────────────────────────

    #[test]
    fn chdata_editor_field_change_updates_catalog() {
        use dispel_core::ChData;

        let mut app = App::test_new(Workspace::new());
        let record = ChData {
            warrior_strength: 10,
            ..Default::default()
        };
        app.state.editors.chdata_editor.state.catalog = Some(vec![record]);
        app.state.editors.chdata_editor.state.refresh();
        app.state.editors.chdata_editor.state.select(0);

        let _task = crate::editors::chdata::handle(
            crate::editors::chdata::ChDataEditorMessage::FieldChanged(
                0,
                "warrior_strength".to_string(),
                "99".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.chdata_editor.state.filtered[0]
                .1
                .warrior_strength,
            99,
            "ChData warrior_strength updated"
        );
    }

    // ── DrawItemEditor (record: DrawItem) ───────────────────────────────────

    #[test]
    fn draw_item_editor_field_change_updates_catalog() {
        use dispel_core::DrawItem;

        let mut app = App::test_new(Workspace::new());
        let record = DrawItem {
            map_id: 1,
            ..Default::default()
        };
        app.state.editors.draw_item_editor.state.catalog = Some(vec![record]);
        app.state.editors.draw_item_editor.state.refresh();
        app.state.editors.draw_item_editor.state.select(0);

        let _task = crate::editors::draw_item::handle(
            crate::editors::draw_item::DrawItemEditorMessage::FieldChanged(
                0,
                "map_id".to_string(),
                "42".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.draw_item_editor.state.filtered[0]
                .1
                .map_id,
            42,
            "DrawItem map_id updated"
        );
    }

    #[test]
    fn draw_item_editor_field_change_oob_is_noop() {
        use dispel_core::DrawItem;

        let mut app = App::test_new(Workspace::new());
        let record = DrawItem {
            map_id: 1,
            ..Default::default()
        };
        app.state.editors.draw_item_editor.state.catalog = Some(vec![record]);
        app.state.editors.draw_item_editor.state.refresh();
        app.state.editors.draw_item_editor.state.select(0);

        let _task = crate::editors::draw_item::handle(
            crate::editors::draw_item::DrawItemEditorMessage::FieldChanged(
                999,
                "map_id".to_string(),
                "42".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.draw_item_editor.state.filtered[0]
                .1
                .map_id,
            1,
            "OOB DrawItem FieldChanged is a no-op"
        );
    }

    // ── EventNpcRefEditor (record: EventNpcRef) ─────────────────────────────

    #[test]
    fn event_npc_ref_editor_field_change_updates_catalog() {
        use dispel_core::EventNpcRef;

        let mut app = App::test_new(Workspace::new());
        let record = EventNpcRef {
            name: "OldName".to_string(),
            ..Default::default()
        };
        app.state.editors.event_npc_ref_editor.state.catalog = Some(vec![record]);
        app.state.editors.event_npc_ref_editor.state.refresh();
        app.state.editors.event_npc_ref_editor.state.select(0);

        let _task = crate::editors::event_npc_ref::handle(
            crate::editors::event_npc_ref::EventNpcRefEditorMessage::FieldChanged(
                0,
                "name".to_string(),
                "NewName".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.event_npc_ref_editor.state.filtered[0]
                .1
                .name,
            "NewName",
            "EventNpcRef name updated"
        );
    }

    #[test]
    fn event_npc_ref_editor_field_change_oob_is_noop() {
        use dispel_core::EventNpcRef;

        let mut app = App::test_new(Workspace::new());
        let record = EventNpcRef {
            name: "OldName".to_string(),
            ..Default::default()
        };
        app.state.editors.event_npc_ref_editor.state.catalog = Some(vec![record]);
        app.state.editors.event_npc_ref_editor.state.refresh();
        app.state.editors.event_npc_ref_editor.state.select(0);

        let _task = crate::editors::event_npc_ref::handle(
            crate::editors::event_npc_ref::EventNpcRefEditorMessage::FieldChanged(
                999,
                "name".to_string(),
                "NewName".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.event_npc_ref_editor.state.filtered[0]
                .1
                .name,
            "OldName",
            "OOB EventNpcRef FieldChanged is a no-op"
        );
    }
}

// ============================================================================
// Custom / non-standard editors
// ============================================================================

mod custom_editors {
    use super::*;

    // ── StoreEditor (custom state, not StandardEditor) ──────────────────────

    #[test]
    fn store_editor_field_change_updates_catalog() {
        use dispel_core::Store;

        let mut app = App::test_new(Workspace::new());
        app.state.editors.store_editor.catalog = Some(vec![Store::default()]);
        app.state.editors.store_editor.refresh_stores();
        app.state.editors.store_editor.select_store(0);

        let _task = crate::editors::store::handle(
            crate::editors::store::StoreEditorMessage::FieldChanged(
                0,
                "store_name".to_string(),
                "New Shop".to_string(),
            ),
            &mut app,
        );

        assert_eq!(
            app.state.editors.store_editor.filtered_stores[0]
                .1
                .store_name,
            "New Shop",
            "Store store_name updated"
        );
    }
}
