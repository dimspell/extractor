//! Exhaustive, agreement-driven tests for the authoritative editor catalog.
//!
//! Historically several test files hand-copied editor lists, so newly added
//! editors were never exercised and a missing dispatch arm went unnoticed.
//! All coverage is now derived from `EditorType::ALL` / `EditorType::iter()`,
//! which are generated from the same catalog declaration as the enum. These
//! tests additionally assert that the command, export, indexation and
//! lifecycle mappings agree with that catalog.

#[cfg(test)]
mod exhaustive {
    use crate::editor_catalog::{EDITOR_COUNT, EditorType};
    use crate::editor_registry::{EditorRegistry, UndoRedoOp};
    use crate::message::{Message, system::SystemMessage};
    use crate::tests::app_with_tab;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn descriptor_table_covers_every_variant() {
        assert_eq!(
            EditorType::ALL.len(),
            EDITOR_COUNT,
            "ALL must list every declared editor exactly once"
        );
        let mut seen = HashSet::new();
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
            assert_eq!(et.supports_undo_redo(), d.lifecycle.has_undo, "{:?}", et);
            assert_eq!(et.has_edit_history(), d.lifecycle.has_history, "{:?}", et);
            assert_eq!(
                crate::editor_catalog::catalog_load_task(et).is_some(),
                d.load.is_some(),
                "{:?}",
                et
            );
        }
    }

    // ── Lifecycle-table agreement ───────────────────────────────────────────

    /// Every lifecycle operation declared by every catalog entry must be
    /// callable on a fresh registry. This is the agreement test for the
    /// tables that used to be hand-maintained in `editor_registry`: a new
    /// editor with a missing/no-op entry is caught here. Spreadsheet caches
    /// are refreshed inside `undo_redo` itself, so there is no separate
    /// refresh operation to exercise here.
    #[test]
    fn every_lifecycle_operation_is_callable_for_every_editor() {
        let lookups = HashMap::new();
        for et in EditorType::iter() {
            let mut registry = EditorRegistry::default();
            let life = et.descriptor().lifecycle;
            let _ = (life.undo_redo)(&mut registry, 0, &lookups, UndoRedoOp::Undo);
            let _ = (life.undo_redo)(&mut registry, 0, &lookups, UndoRedoOp::Redo);
            (life.remove_tab)(&mut registry, 0);
            (life.clear_tabs)(&mut registry);
            let _ = (life.edit_history)(&registry, 0);
            (life.clear)(&mut registry);
        }
    }

    #[test]
    fn clear_all_resets_boxed_and_tabbed_state() {
        let mut registry = EditorRegistry::default();
        registry.weapon_editor.catalog = Some(vec![]);
        registry.fog_editors.insert(7, Default::default());
        registry.save_file_viewers.insert(8, Default::default());
        registry.clear_all();
        assert!(registry.weapon_editor.catalog.is_none());
        assert!(registry.fog_editors.is_empty());
        assert!(registry.save_file_viewers.is_empty());
    }

    #[test]
    fn close_all_tabs_preserves_boxed_state() {
        let mut registry = EditorRegistry::default();
        registry.weapon_editor.catalog = Some(vec![]);
        registry.fog_editors.insert(7, Default::default());
        registry.close_all_tabs();
        assert!(
            registry.weapon_editor.catalog.is_some(),
            "boxed editor state must survive CloseAll"
        );
        assert!(registry.fog_editors.is_empty());
    }

    // ── Catalog-derived mapping agreement ───────────────────────────────────

    /// Scan/save is reached from each editor's own buttons and global Ctrl+S,
    /// so the palette must not repeat those actions. The catalog still owns the
    /// load/save messages those buttons send; the remaining palette commands
    /// keep their editor scoping.
    #[test]
    fn command_applicability_agrees_with_catalog() {
        let mut load_by_msg: HashMap<String, EditorType> = HashMap::new();
        let mut save_by_msg: HashMap<String, EditorType> = HashMap::new();
        for et in EditorType::iter() {
            let d = et.descriptor();
            if let Some(load) = d.load {
                load_by_msg.insert(format!("{:?}", load()), et);
            }
            if let Some(save) = d.save {
                save_by_msg.insert(format!("{:?}", save(0)), et);
            }
        }
        assert!(
            load_by_msg.len() >= 20 && save_by_msg.len() >= 20,
            "the catalog must still declare load/save messages for per-editor buttons"
        );

        let mut scoped = 0;
        for cmd in crate::components::command_palette::Command::all() {
            let is_scan_save = cmd.id.starts_with("scan-")
                || cmd.id.starts_with("load-")
                || cmd.id.starts_with("save-");
            assert!(
                !is_scan_save,
                "palette must not duplicate a per-editor scan/save action: {}",
                cmd.id
            );
            let _ = (cmd.action)();
            if cmd.id.starts_with("hex-") {
                assert_eq!(
                    cmd.applicable_editors,
                    vec![EditorType::HexEditor],
                    "hex commands must stay scoped to the hex editor"
                );
                scoped += 1;
            } else {
                assert!(
                    cmd.applicable_editors.is_empty(),
                    "command '{}' should be universal, got {:?}",
                    cmd.id,
                    cmd.applicable_editors
                );
            }
        }
        assert!(
            scoped >= 2,
            "expected the hex editor commands to remain in the palette, found {scoped}"
        );
    }

    /// JSON export coverage must come from the catalog, not a second match.
    #[test]
    fn export_coverage_agrees_with_catalog() {
        let extractable: Vec<EditorType> = EditorType::iter()
            .filter(|et| et.descriptor().extract.is_some())
            .collect();
        assert!(
            extractable.len() >= 30,
            "expected catalog to declare many extractable editors, got {}",
            extractable.len()
        );
        // Representative parsers that used to be in the hand-written match.
        for et in [
            EditorType::WeaponEditor,
            EditorType::AllMapIniEditor,
            EditorType::MapEditor,
            EditorType::TilesetEditor,
            EditorType::SpriteViewer,
        ] {
            assert!(
                et.descriptor().extract.is_some(),
                "EditorType::{:?} should declare JSON export",
                et
            );
        }
        // Editors with no file parser must not claim export support.
        for et in [EditorType::HexEditor, EditorType::ModPackager] {
            assert!(
                et.descriptor().extract.is_none(),
                "EditorType::{:?} should not declare JSON export",
                et
            );
        }
    }

    /// Extension→editor names used by the search index must be real catalog
    /// names, never invented strings like "DatabaseEditor" or "IniEditor".
    #[test]
    fn indexation_editor_names_are_catalog_names() {
        let names: HashSet<&str> = EditorType::iter().map(|et| et.name()).collect();
        for file in [
            "WeaponItem.db",
            "Monster.ini",
            "Mondun00.ref",
            "quest.scr",
            "Some.map",
            "tiles.gtl",
            "hero.spr",
            "effect.snf",
            "unknownext.xyz",
        ] {
            let et = EditorType::from_path(std::path::Path::new(file));
            assert!(
                names.contains(et.name()),
                "editor name '{}' for '{}' is not in the catalog",
                et.name(),
                file
            );
        }
        assert!(!names.contains("DatabaseEditor"));
        assert!(!names.contains("IniEditor"));
    }

    #[test]
    fn from_catalog_editor_is_derived_from_catalog_names() {
        for et in EditorType::iter() {
            assert_eq!(
                crate::editor_catalog::from_catalog_editor(et.name()),
                Some(et),
                "catalog id '{}' must round-trip",
                et.name()
            );
        }
        assert_eq!(
            crate::editor_catalog::from_catalog_editor("NoSuchEditor"),
            None
        );
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
