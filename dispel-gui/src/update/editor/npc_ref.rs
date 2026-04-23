// NpcRef editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages_tab;
use crate::loading_state::LoadingState;
use crate::message::editor::npc_ref::NpcRefEditorMessage;
use crate::message::MessageExt;
use dispel_core::Extractor;
use iced::Task;

pub fn handle(message: NpcRefEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = app
        .state
        .workspace
        .active()
        .map(|t| t.id)
        .unwrap_or(usize::MAX);

    match message {
        NpcRefEditorMessage::Select(index) => {
            if let Some(editor) = app.state.npc_ref_editors.get_mut(&tab_id) {
                editor.select_npc(index);
            }
            Task::none()
        }
        NpcRefEditorMessage::FieldChanged(index, field, value) => {
            if let Some(editor) = app.state.npc_ref_editors.get_mut(&tab_id) {
                editor.update_field(index, &field, value);
            }
            Task::none()
        }
        NpcRefEditorMessage::Save => {
            if let Some(editor) = app.state.npc_ref_editors.get_mut(&tab_id) {
                editor.editor.loading_state = LoadingState::Loading;
                let result = editor.save_npcs();
                editor.editor.loading_state = LoadingState::Loaded(());
                match result {
                    Ok(_) => editor.editor.status_msg = "NPC refs saved successfully.".into(),
                    Err(e) => editor.editor.status_msg = format!("Error saving NPC refs: {}", e),
                }
            }
            Task::none()
        }
        NpcRefEditorMessage::AddEntry => {
            if let Some(editor) = app.state.npc_ref_editors.get_mut(&tab_id) {
                editor.add_record();
            }
            Task::none()
        }
        NpcRefEditorMessage::RemoveEntry(index) => {
            if let Some(editor) = app.state.npc_ref_editors.get_mut(&tab_id) {
                editor.remove_record(index);
            }
            Task::none()
        }
        NpcRefEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages_tab!(
                app,
                npc_ref_spreadsheets,
                npc_ref_editors,
                &tab_id,
                |index, field, value| crate::message::Message::npc_ref(
                    NpcRefEditorMessage::FieldChanged(index, field, value)
                ),
                msg
            );
            Task::none()
        }
        NpcRefEditorMessage::PaneResized(event) => {
            if let Some(ed) = app.state.npc_ref_editors.get_mut(&tab_id) {
                if let Some(ref mut ps) = ed.editor.pane_state {
                    ps.resize(event.split, event.ratio);
                }
            }
            if let Some(ss) = app.state.npc_ref_spreadsheets.get_mut(&tab_id) {
                if let Some(ref mut ps) = ss.pane_state {
                    ps.resize(event.split, event.ratio);
                }
            }
            Task::none()
        }
        NpcRefEditorMessage::PaneClicked(pane) => {
            if let Some(ed) = app.state.npc_ref_editors.get_mut(&tab_id) {
                ed.editor.pane_focus = Some(pane);
            }
            Task::none()
        }
        NpcRefEditorMessage::LoadCatalog(path) => {
            let tab_id = app
                .state
                .workspace
                .active()
                .map(|t| t.id)
                .unwrap_or(usize::MAX);

            let mut editor_state = crate::state::npc_ref_editor::NpcRefEditorState::default();
            editor_state.current_file = Some(path.clone());

            // Load catalog first
            editor_state.select_file(path.clone());

            // Initialize spreadsheet state with the loaded catalog
            let mut ss = crate::view::editor::SpreadsheetState::new();
            if let Some(catalog) = editor_state.editor.catalog.as_ref() {
                ss.apply_filter(catalog);
                ss.compute_all_caches(catalog);
                ss.init_pane_state();
            }

            app.state.npc_ref_editors.insert(tab_id, editor_state);
            app.state.npc_ref_spreadsheets.insert(tab_id, ss);

            // Load NPC names
            if !app.state.lookups.contains_key("NPC") {
                let game_path = app.state.shared_game_path.clone();
                return Task::perform(
                    async move {
                        let path = std::path::PathBuf::from(&game_path).join("Npc.ini");
                        dispel_core::NpcIni::read_file(&path)
                            .map(|npcs| {
                                npcs.iter()
                                    .map(|n| (n.id.to_string(), n.description.clone()))
                                    .collect()
                            })
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    move |result| {
                        crate::message::Message::npc_ref(NpcRefEditorMessage::NpcNamesLoaded(
                            result,
                        ))
                    },
                );
            }

            Task::none()
        }
        NpcRefEditorMessage::NpcNamesLoaded(result) => {
            if let Ok(names) = result {
                if let Some(_editor) = app.state.npc_ref_editors.get_mut(&tab_id) {
                    app.state.lookups.insert("NPC".to_string(), names);
                }
            }
            Task::none()
        }
    }
}
