#[cfg(test)]
mod tabbed_editor_lifecycle_tests {
    use crate::app::App;
    use crate::workspace::{EditorType, Workspace};
    use std::path::PathBuf;

    fn app_with_tabbed_editor(editor_type: EditorType) -> App {
        let mut app = App::test_new(Workspace::new());
        app.state
            .workspace
            .open("test.dlg".into(), Some(PathBuf::from("test.dlg")));
        if let Some(tab) = app.state.workspace.tabs.last_mut() {
            tab.editor_type = editor_type;
        }
        let tab_id = app.state.workspace.active().unwrap().id;
        // Insert a dummy entry for each tabbed editor type
        match editor_type {
            EditorType::MonsterRefEditor => {
                app.state
                    .editors
                    .monster_ref_editor
                    .editors
                    .insert(tab_id, Default::default());
                app.state
                    .editors
                    .monster_ref_editor
                    .spreadsheets
                    .insert(tab_id, Default::default());
            }
            EditorType::NpcRefEditor => {
                app.state
                    .editors
                    .npc_ref_editor
                    .editors
                    .insert(tab_id, Default::default());
                app.state
                    .editors
                    .npc_ref_editor
                    .spreadsheets
                    .insert(tab_id, Default::default());
            }
            EditorType::ExtraRefEditor => {
                app.state
                    .editors
                    .extra_ref_editor
                    .editors
                    .insert(tab_id, Default::default());
                app.state
                    .editors
                    .extra_ref_editor
                    .spreadsheets
                    .insert(tab_id, Default::default());
            }
            EditorType::DialogueScriptEditor => {
                app.state
                    .editors
                    .dialogue_script_editor
                    .editors
                    .insert(tab_id, Default::default());
                app.state
                    .editors
                    .dialogue_script_editor
                    .spreadsheets
                    .insert(tab_id, Default::default());
            }
            EditorType::DialogueTextEditor => {
                app.state
                    .editors
                    .dialogue_paragraph_editor
                    .editors
                    .insert(tab_id, Default::default());
                app.state
                    .editors
                    .dialogue_paragraph_editor
                    .spreadsheets
                    .insert(tab_id, Default::default());
            }
            _ => {}
        }
        app
    }

    #[test]
    fn remove_tab_clears_tabbed_editor_entries() {
        let mut app = app_with_tabbed_editor(EditorType::MonsterRefEditor);
        let tab_id = app.state.workspace.active().unwrap().id;
        assert!(
            !app.state.editors.monster_ref_editor.editors.is_empty(),
            "precondition: editor entry exists before remove"
        );
        assert!(
            !app.state.editors.monster_ref_editor.spreadsheets.is_empty(),
            "precondition: spreadsheet entry exists before remove"
        );

        app.state.editors.remove_tab(tab_id);

        assert!(
            app.state.editors.monster_ref_editor.editors.is_empty(),
            "editor entry removed"
        );
        assert!(
            app.state.editors.monster_ref_editor.spreadsheets.is_empty(),
            "spreadsheet entry removed"
        );
    }

    #[test]
    fn remove_tab_clears_all_tabbed_editor_types() {
        let editors = [
            EditorType::MonsterRefEditor,
            EditorType::NpcRefEditor,
            EditorType::ExtraRefEditor,
            EditorType::DialogueScriptEditor,
            EditorType::DialogueTextEditor,
        ];
        for et in editors {
            let mut app = app_with_tabbed_editor(et);
            let tab_id = app.state.workspace.active().unwrap().id;
            app.state.editors.remove_tab(tab_id);
            // Map editor types to their field names to verify cleanup
            match et {
                EditorType::MonsterRefEditor => {
                    assert!(
                        app.state.editors.monster_ref_editor.editors.is_empty(),
                        "MonsterRefEditor editors not cleaned up for tab_id {tab_id}"
                    );
                }
                EditorType::NpcRefEditor => {
                    assert!(
                        app.state.editors.npc_ref_editor.editors.is_empty(),
                        "NpcRefEditor editors not cleaned up"
                    );
                }
                EditorType::ExtraRefEditor => {
                    assert!(
                        app.state.editors.extra_ref_editor.editors.is_empty(),
                        "ExtraRefEditor editors not cleaned up"
                    );
                }
                EditorType::DialogueScriptEditor => {
                    assert!(
                        app.state.editors.dialogue_script_editor.editors.is_empty(),
                        "DialogueScriptEditor editors not cleaned up"
                    );
                }
                EditorType::DialogueTextEditor => {
                    assert!(
                        app.state
                            .editors
                            .dialogue_paragraph_editor
                            .editors
                            .is_empty(),
                        "DialogueTextEditor editors not cleaned up"
                    );
                }
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn remove_tab_nonexistent_id_does_not_panic() {
        let mut app = App::test_new(Workspace::new());
        app.state.editors.remove_tab(999); // should not panic
    }

    #[test]
    fn undo_active_nonexistent_tabbed_editor_returns_none() {
        let et = EditorType::MonsterRefEditor;
        let mut app = app_with_tabbed_editor(et);
        // Use a tab_id that doesn't match any entry
        let result = app.state.editors.undo_active(et, 999, &Default::default());
        assert!(
            result.is_none(),
            "undo on nonexistent tabbed editor returns None"
        );
    }

    #[test]
    fn redo_active_nonexistent_tabbed_editor_returns_none() {
        let et = EditorType::MonsterRefEditor;
        let mut app = app_with_tabbed_editor(et);
        let result = app.state.editors.redo_active(et, 999, &Default::default());
        assert!(
            result.is_none(),
            "redo on nonexistent tabbed editor returns None"
        );
    }
}

#[cfg(test)]
mod tabbed_editor_catalog_tests {
    use crate::app::App;
    use crate::workspace::{EditorType, Workspace};
    use std::path::PathBuf;

    /// Exercise the shared catalog-load flow for one tabbed editor type.
    ///
    /// Checks: async read spawned, path recorded, loading state transitions,
    /// catalog installed in the right tab, and spreadsheet caches rebuilt.
    macro_rules! assert_catalog_load {
        ($name:ident, $editor_type:expr, $handle:path, $Msg:ty, $field:ident, $path:expr, $record:ty) => {
            #[test]
            fn $name() {
                let mut app = App::test_new(Workspace::new());
                app.state.workspace.open("catalog.bin".into(), None);
                if let Some(tab) = app.state.workspace.tabs.last_mut() {
                    tab.editor_type = $editor_type;
                }
                let tab_id = app.state.workspace.active().unwrap().id;

                // `LoadCatalog` spawns exactly one async read and records the
                // file path immediately (no second, synchronous parse).
                let task = $handle(<$Msg>::LoadCatalog(PathBuf::from($path)), &mut app);
                assert!(task.units() > 0, "catalog read spawned");
                let editor = app.state.editors.$field.editors.get(&tab_id).unwrap();
                assert_eq!(
                    editor.current_file.as_deref(),
                    Some(std::path::Path::new($path))
                );
                assert!(editor.editor.loading_state.is_loading());

                // Deliver the completion for the captured tab.
                let catalog: Vec<$record> = vec![Default::default(), Default::default()];
                let _ = $handle(<$Msg>::CatalogLoaded(tab_id, Ok(catalog)), &mut app);

                let editor = app.state.editors.$field.editors.get(&tab_id).unwrap();
                assert!(editor.editor.loading_state.is_loaded());
                assert_eq!(editor.editor.catalog.as_ref().unwrap().len(), 2);
                assert_eq!(editor.editor.filtered.len(), 2, "record list refreshed");

                let ss = app.state.editors.$field.spreadsheets.get(&tab_id).unwrap();
                assert_eq!(ss.filtered_indices, vec![0, 1], "filter cache rebuilt");
                assert_eq!(ss.row_hashes.len(), 2, "display caches rebuilt");
                assert!(ss.pane_state.is_some(), "pane state initialised");
            }
        };
    }

    assert_catalog_load!(
        monster_ref_loads_catalog_into_its_tab,
        EditorType::MonsterRefEditor,
        crate::editors::monster_ref::handle,
        crate::editors::monster_ref::MonsterRefEditorMessage,
        monster_ref_editor,
        "Mondun01.ref",
        dispel_core::MonsterRef
    );
    assert_catalog_load!(
        npc_ref_loads_catalog_into_its_tab,
        EditorType::NpcRefEditor,
        crate::editors::npc_ref::handle,
        crate::editors::npc_ref::NpcRefEditorMessage,
        npc_ref_editor,
        "Npccat1.ref",
        dispel_core::NPC
    );
    assert_catalog_load!(
        extra_ref_loads_catalog_into_its_tab,
        EditorType::ExtraRefEditor,
        crate::editors::extra_ref::handle,
        crate::editors::extra_ref::ExtraRefEditorMessage,
        extra_ref_editor,
        "Ext01.ref",
        dispel_core::ExtraRef
    );
    assert_catalog_load!(
        dialogue_script_loads_catalog_into_its_tab,
        EditorType::DialogueScriptEditor,
        crate::editors::dialogue_script::handle,
        crate::editors::dialogue_script::DialogueScriptEditorMessage,
        dialogue_script_editor,
        "script.dlg",
        dispel_core::DialogueScript
    );
    assert_catalog_load!(
        dialogue_paragraph_loads_catalog_into_its_tab,
        EditorType::DialogueTextEditor,
        crate::editors::dialogue_paragraph::handle,
        crate::editors::dialogue_paragraph::DialogueParagraphEditorMessage,
        dialogue_paragraph_editor,
        "text.pgp",
        dispel_core::DialogueParagraph
    );

    #[test]
    fn catalog_completion_lands_in_captured_tab_after_switch() {
        // Regression: the async result must be routed by the tab id captured
        // at spawn time, never by whichever tab happens to be active when the
        // message arrives.
        use crate::editors::dialogue_script::DialogueScriptEditorMessage;

        let mut app = App::test_new(Workspace::new());
        app.state
            .workspace
            .open("a.dlg".into(), Some(PathBuf::from("a.dlg")));
        let tab_a = app.state.workspace.active().unwrap().id;
        let idx_a = app.state.workspace.active_index().unwrap();
        app.state
            .workspace
            .open("b.dlg".into(), Some(PathBuf::from("b.dlg")));
        let tab_b = app.state.workspace.active().unwrap().id;
        let idx_b = app.state.workspace.active_index().unwrap();
        for tab in [tab_a, tab_b] {
            app.state
                .editors
                .dialogue_script_editor
                .editors
                .insert(tab, Default::default());
            app.state
                .editors
                .dialogue_script_editor
                .spreadsheets
                .insert(tab, Default::default());
        }

        // Start the load for A while A is active.
        app.state.workspace.active_tab = Some(idx_a);
        let task = crate::editors::dialogue_script::handle(
            DialogueScriptEditorMessage::LoadCatalog(PathBuf::from("a.dlg")),
            &mut app,
        );
        assert!(task.units() > 0, "catalog read spawned for tab A");

        // Switch to B before the read completes.
        app.state.workspace.active_tab = Some(idx_b);

        // Completion carries A's tab id.
        let _ = crate::editors::dialogue_script::handle(
            DialogueScriptEditorMessage::CatalogLoaded(tab_a, Ok(vec![Default::default()])),
            &mut app,
        );

        assert!(
            app.state
                .editors
                .dialogue_script_editor
                .editors
                .get(&tab_a)
                .unwrap()
                .editor
                .catalog
                .is_some(),
            "catalog landed in tab A"
        );
        assert!(
            app.state
                .editors
                .dialogue_script_editor
                .editors
                .get(&tab_b)
                .unwrap()
                .editor
                .catalog
                .is_none(),
            "tab B was not overwritten"
        );
    }

    #[test]
    fn catalog_completion_for_closed_tab_is_safely_dropped() {
        use crate::editors::dialogue_paragraph::DialogueParagraphEditorMessage;

        let mut app = App::test_new(Workspace::new());
        app.state.workspace.open("text.pgp".into(), None);
        let tab_id = app.state.workspace.active().unwrap().id;
        app.state
            .editors
            .dialogue_paragraph_editor
            .editors
            .insert(tab_id, Default::default());
        app.state
            .editors
            .dialogue_paragraph_editor
            .spreadsheets
            .insert(tab_id, Default::default());

        // Simulate the tab being closed while the read was in flight.
        app.state.editors.dialogue_paragraph_editor.remove(&tab_id);

        let _ = crate::editors::dialogue_paragraph::handle(
            DialogueParagraphEditorMessage::CatalogLoaded(tab_id, Ok(vec![Default::default()])),
            &mut app,
        );

        assert!(
            !app.state
                .editors
                .dialogue_paragraph_editor
                .contains_key(&tab_id),
            "stale result must not resurrect a closed tab's editor"
        );
    }

    #[test]
    fn unhandled_extension_variant_is_a_noop_not_a_panic() {
        // `handle_core` deliberately does not handle extension variants; they
        // are routed by `handle`. Reaching the catch-all must be harmless.
        use crate::editors::monster_ref::{MonsterRefEditorMessage, MonsterRefExtension};

        let mut app = App::test_new(Workspace::new());
        let task = crate::editors::monster_ref::handle_core(
            MonsterRefEditorMessage::Extension(MonsterRefExtension::LoadMonsterNames),
            &mut app,
            0,
        );
        assert_eq!(task.units(), 0, "unhandled variant is a no-op");
    }

    #[test]
    fn undo_refreshes_tabbed_editor_spreadsheet_caches() {
        // The unified undo path must rebuild caches for the edited tab in the
        // same operation (no separate `refresh_spreadsheet` pass).
        use crate::components::generic_editor::MultiFileEditorState;
        use crate::view::editor::SpreadsheetState;

        let mut app = App::test_new(Workspace::new());
        app.state
            .workspace
            .open("Mondun01.ref".into(), Some(PathBuf::from("Mondun01.ref")));
        let tab_id = app.state.workspace.active().unwrap().id;

        let record = dispel_core::MonsterRef::default();
        let mut editor = MultiFileEditorState::default();
        editor.editor.catalog = Some(vec![record.clone()]);
        editor.editor.filtered = vec![(0, record)];
        assert!(
            editor.editor.update_field(0, "placement_id", "42".into()),
            "edit creates undo history"
        );
        app.state
            .editors
            .monster_ref_editor
            .editors
            .insert(tab_id, editor);
        app.state
            .editors
            .monster_ref_editor
            .spreadsheets
            .insert(tab_id, SpreadsheetState::new());
        app.state
            .editors
            .monster_ref_editor
            .spreadsheets
            .get_mut(&tab_id)
            .unwrap()
            .row_hashes
            .clear();

        let result = app.state.undo_active(EditorType::MonsterRefEditor, tab_id);
        assert!(result.is_some(), "undo performed");
        assert_eq!(
            app.state
                .editors
                .monster_ref_editor
                .spreadsheets
                .get(&tab_id)
                .unwrap()
                .row_hashes
                .len(),
            1,
            "spreadsheet caches rebuilt by the undo itself"
        );
    }
}

#[cfg(test)]
mod wave_ini_extension_tests {
    use crate::app::App;
    use crate::editors::wave_ini::{WaveIniEditorMessage, WaveIniExtension};
    use crate::workspace::Workspace;

    #[test]
    fn wave_ini_export_without_game_path_is_a_noop() {
        let mut app = App::test_new(Workspace::new());
        app.state.editors.wave_ini_editor.filtered = vec![(
            0,
            dispel_core::WaveIni {
                snf_filename: Some("test.snf".into()),
                ..Default::default()
            },
        )];

        let task = crate::editors::wave_ini::handle(
            WaveIniEditorMessage::Extension(WaveIniExtension::ExportWav(0)),
            &mut app,
        );

        assert_eq!(task.units(), 0);
        assert_eq!(
            app.state.editors.wave_ini_editor.status_msg,
            "Please select game path first."
        );
    }

    #[test]
    fn wave_ini_export_without_snf_filename_is_a_noop() {
        let mut app = App::test_new(Workspace::new());
        app.state.shared_game_path = "/game".into();
        app.state.editors.wave_ini_editor.filtered = vec![(0, dispel_core::WaveIni::default())];

        let task = crate::editors::wave_ini::handle(
            WaveIniEditorMessage::Extension(WaveIniExtension::ExportWav(0)),
            &mut app,
        );

        assert_eq!(task.units(), 0);
        assert_eq!(
            app.state.editors.wave_ini_editor.status_msg,
            "No SNF filename for this entry."
        );
    }

    #[test]
    fn wave_ini_export_completion_updates_status() {
        let mut app = App::test_new(Workspace::new());

        let _ = crate::editors::wave_ini::handle(
            WaveIniEditorMessage::Extension(WaveIniExtension::ExportedWav(Ok(
                "/tmp/out.wav".into()
            ))),
            &mut app,
        );

        assert_eq!(
            app.state.editors.wave_ini_editor.status_msg,
            "Exported to /tmp/out.wav"
        );
        assert!(app.state.editors.wave_ini_editor.loading_state.is_loaded());
    }
}
