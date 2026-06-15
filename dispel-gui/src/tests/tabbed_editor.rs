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

    #[test]
    fn refresh_spreadsheet_stale_tab_id_is_noop() {
        let et = EditorType::MonsterRefEditor;
        let mut app = app_with_tabbed_editor(et);
        app.state
            .editors
            .refresh_spreadsheet(et, 999, &Default::default());
        // should not panic
    }
}

#[cfg(test)]
mod tabbed_editor_message_routing_tests {
    use crate::app::App;
    use crate::editors::dialogue_paragraph::DialogueParagraphEditorMessage;
    use crate::editors::dialogue_script::DialogueScriptEditorMessage;
    use crate::editors::extra_ref::ExtraRefEditorMessage;
    use crate::editors::monster_ref::MonsterRefEditorMessage;
    use crate::editors::npc_ref::NpcRefEditorMessage;
    use crate::view::editor::SpreadsheetMessage;
    use crate::workspace::Workspace;

    #[test]
    fn monster_ref_message_unknown_tab_is_noop() {
        let mut app = App::test_new(Workspace::new());
        // No active tab → handle_core receives usize::MAX as tab_id → no-op
        let msg = MonsterRefEditorMessage::Spreadsheet(SpreadsheetMessage::SelectRow(0));
        let task = crate::editors::monster_ref::handle(msg, &mut app);
        assert_eq!(task.units(), 0, "no-op with no active tab");
    }

    #[test]
    fn npc_ref_message_unknown_tab_is_noop() {
        let mut app = App::test_new(Workspace::new());
        let msg = NpcRefEditorMessage::Spreadsheet(SpreadsheetMessage::SelectRow(0));
        let task = crate::editors::npc_ref::handle(msg, &mut app);
        assert_eq!(task.units(), 0, "no-op with no active tab");
    }

    #[test]
    fn extra_ref_message_unknown_tab_is_noop() {
        let mut app = App::test_new(Workspace::new());
        let msg = ExtraRefEditorMessage::Spreadsheet(SpreadsheetMessage::SelectRow(0));
        let task = crate::editors::extra_ref::handle(msg, &mut app);
        assert_eq!(task.units(), 0, "no-op with no active tab");
    }

    #[test]
    fn dialogue_script_message_unknown_tab_is_noop() {
        let mut app = App::test_new(Workspace::new());
        let msg = DialogueScriptEditorMessage::Spreadsheet(SpreadsheetMessage::SelectRow(0));
        let task = crate::editors::dialogue_script::handle(msg, &mut app);
        assert_eq!(task.units(), 0, "no-op with no active tab");
    }

    #[test]
    fn dialogue_paragraph_message_unknown_tab_is_noop() {
        let mut app = App::test_new(Workspace::new());
        let msg = DialogueParagraphEditorMessage::Spreadsheet(SpreadsheetMessage::SelectRow(0));
        let task = crate::editors::dialogue_paragraph::handle(msg, &mut app);
        assert_eq!(task.units(), 0, "no-op with no active tab");
    }
}
