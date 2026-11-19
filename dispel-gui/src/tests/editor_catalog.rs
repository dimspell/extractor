//! Exhaustive tests driven by the authoritative editor descriptor table.
//!
//! Historically three separate test files each hand-copied a (stale) list of
//! editor variants, so newly added editors were never exercised and a missing
//! dispatch arm could crash the app or silently disable undo. All three lists
//! are replaced by iteration over `EditorType::ALL`, which is generated from
//! the very same catalog declaration as the enum itself.

#[cfg(test)]
mod exhaustive {
    use crate::editor_catalog::{EDITOR_COUNT, EditorType};
    use crate::editor_registry::EditorRegistry;
    use crate::message::{Message, system::SystemMessage};
    use crate::tests::app_with_tab;
    use std::collections::HashMap;

    /// Compile-time exhaustiveness guard: this match has no wildcard, so
    /// adding a variant forces an explicit, reviewed decision here. In normal
    /// development `EditorType::ALL` is regenerated automatically from the
    /// same catalog declaration as the enum.
    #[allow(dead_code)]
    fn assert_all_variants_enumerated(et: EditorType) {
        use EditorType::*;
        match et {
            WeaponEditor | MonsterEditor | MonsterIniEditor | HealItemEditor | MiscItemEditor
            | EditItemEditor | EventItemEditor | MagicEditor | StoreEditor | ChDataEditor
            | PartyLevelDbEditor | DialogueScriptEditor | DialogueTextEditor | DrawItemEditor
            | EventIniEditor | EventNpcRefEditor | ExtraIniEditor | ExtraRefEditor
            | MapIniEditor | MessageScrEditor | MonsterRefEditor | NpcIniEditor | NpcRefEditor
            | PartyRefEditor | PartyIniEditor | QuestScrEditor | EventScrEditor | WaveIniEditor
            | AllMapIniEditor | SpriteViewer | SnfEditor | DbViewer | TilesetEditor | MapEditor
            | ModPackager | LocalizationManager | SaveFileViewer | SaveIfoEditor
            | FogDataEditor | ChestSpawnEditor | MonsterSpawnEditor | MapInfoMultiEditor
            | TileGridEditor | HexEditor | Unknown => {}
        }
    }

    #[test]
    fn descriptor_table_covers_every_variant() {
        assert_eq!(
            EditorType::ALL.len(),
            EDITOR_COUNT,
            "ALL must list every declared editor exactly once"
        );
        let mut seen = std::collections::HashSet::new();
        for (index, et) in EditorType::ALL.iter().copied().enumerate() {
            assert_eq!(
                et as usize, index,
                "enum discriminant order must match ALL for {:?}",
                et
            );
            assert!(seen.insert(et), "duplicate editor variant in ALL: {:?}", et);
            assert_eq!(
                et.descriptor().editor_type,
                et,
                "descriptor table index mismatch for {:?}",
                et
            );
            assert_all_variants_enumerated(et);
        }
    }

    #[test]
    fn save_contract_holds_for_all_types() {
        for et in EditorType::iter() {
            let mut app = app_with_tab(et);
            let _task = app.update(Message::System(SystemMessage::Save));
            if et.supports_save() {
                assert_ne!(
                    app.state.status_msg, "This editor does not support saving",
                    "EditorType::{:?} supports_save() but Save rejected it",
                    et
                );
                assert!(
                    crate::editor_catalog::save_task_for_editor(et, 1).is_some(),
                    "EditorType::{:?} supports_save() but has no save message",
                    et
                );
            } else {
                assert_eq!(
                    app.state.status_msg, "This editor does not support saving",
                    "EditorType::{:?} !supports_save() but Save was accepted",
                    et
                );
                assert!(
                    crate::editor_catalog::save_task_for_editor(et, 1).is_none(),
                    "EditorType::{:?} !supports_save() but has a save message",
                    et
                );
            }
        }
    }

    #[test]
    fn undo_active_does_not_panic_for_any_type() {
        let mut registry = EditorRegistry::default();
        let lookups = HashMap::new();
        for et in EditorType::iter() {
            let result = registry.undo_active(et, 0, &lookups);
            assert!(
                result.is_none(),
                "EditorType::{:?} undo_active returned Some when history is empty",
                et
            );
        }
    }

    #[test]
    fn redo_active_does_not_panic_for_any_type() {
        let mut registry = EditorRegistry::default();
        let lookups = HashMap::new();
        for et in EditorType::iter() {
            let result = registry.redo_active(et, 0, &lookups);
            assert!(
                result.is_none(),
                "EditorType::{:?} redo_active returned Some when history is empty",
                et
            );
        }
    }

    #[test]
    fn edit_history_does_not_panic_for_any_type() {
        let registry = EditorRegistry::default();
        for et in EditorType::iter() {
            let result = registry.get_active_edit_history(et, 0);
            if !et.has_edit_history() {
                assert!(
                    result.is_none(),
                    "EditorType::{:?} !has_edit_history() but get_active_edit_history returned Some",
                    et
                );
            }
        }
    }

    #[test]
    fn capability_flags_match_descriptor() {
        for et in EditorType::iter() {
            let d = et.descriptor();
            assert_eq!(et.supports_save(), d.save.is_some(), "{:?}", et);
            assert_eq!(et.auto_load_catalog(), d.load.is_some(), "{:?}", et);
            assert_eq!(
                crate::editor_catalog::catalog_load_task(et).is_some(),
                d.load.is_some(),
                "{:?}",
                et
            );
        }
    }

    #[test]
    fn tabbed_editor_edit_history_available_after_insert() {
        let mut registry = EditorRegistry::default();
        let tab_id = 42;
        registry
            .npc_ref_editor
            .editors
            .insert(tab_id, Default::default());
        let history = registry.get_active_edit_history(EditorType::NpcRefEditor, tab_id);
        assert!(
            history.is_some(),
            "NpcRefEditor should have edit history after insert"
        );
    }

    #[test]
    fn test_save_sprite_viewer_returns_task() {
        let mut app = app_with_tab(EditorType::SpriteViewer);
        let _task = app.update(Message::System(SystemMessage::Save));
        assert_ne!(app.state.status_msg, "This editor does not support saving");
    }

    #[test]
    fn test_save_no_active_tab_shows_correct_status() {
        let mut app = crate::app::App::test_new(crate::workspace::Workspace::new());
        let _ = app.update(Message::System(SystemMessage::Save));
        assert_eq!(app.state.status_msg, "No active tab to save");
    }

    #[cfg(feature = "iced_test")]
    #[test]
    fn all_editor_types_render_without_panic() {
        use iced_test::simulator;
        for et in EditorType::iter() {
            let app = app_with_tab(et);
            let view = app.view();
            // iced_test runs widget layout, catching layout panics.
            let _ui = simulator(view);
        }
    }
}
