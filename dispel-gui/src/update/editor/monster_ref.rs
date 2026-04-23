// MonsterRef editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages_tab;
use crate::message::editor::monster_ref::MonsterRefEditorMessage;
use crate::message::MessageExt;
use dispel_core::{Extractor, MonsterIni};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: MonsterRefEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = app
        .state
        .workspace
        .active()
        .map(|t| t.id)
        .unwrap_or(usize::MAX);

    match message {
        MonsterRefEditorMessage::SelectEntry(index) => {
            if let Some(editor) = app.state.monster_ref_editors.get_mut(&tab_id) {
                editor.select(index);
            }
            Task::none()
        }
        MonsterRefEditorMessage::AddEntry => {
            if let Some(editor) = app.state.monster_ref_editors.get_mut(&tab_id) {
                editor.add_record();
            }
            Task::none()
        }
        MonsterRefEditorMessage::RemoveEntry(index) => {
            if let Some(editor) = app.state.monster_ref_editors.get_mut(&tab_id) {
                editor.remove_record(index);
            }
            Task::none()
        }
        MonsterRefEditorMessage::FieldChanged(index, field, value) => {
            if let Some(editor) = app.state.monster_ref_editors.get_mut(&tab_id) {
                editor.update_field(index, &field, value);
            }
            Task::none()
        }
        MonsterRefEditorMessage::Save => {
            if let Some(editor) = app.state.monster_ref_editors.get_mut(&tab_id) {
                editor.editor.loading_state = crate::loading_state::LoadingState::Loading;
                let result = editor.save();
                editor.editor.loading_state = crate::loading_state::LoadingState::Loaded(());
                match result {
                    Ok(_) => editor.editor.status_msg = "Monster ref saved successfully.".into(),
                    Err(e) => editor.editor.status_msg = format!("Error saving monster ref: {}", e),
                }
            }
            Task::none()
        }
        MonsterRefEditorMessage::LoadMonsterNames => {
            if app.state.shared_game_path.is_empty() {
                return Task::none();
            }
            let path = PathBuf::from(&app.state.shared_game_path).join("Monster.ini");
            Task::perform(
                async move {
                    MonsterIni::read_file(&path)
                        .map(|monsters| {
                            monsters
                                .iter()
                                .map(|m| (m.id.to_string(), m.name.clone().unwrap_or_default()))
                                .collect()
                        })
                        .map_err(|e| e.to_string())
                },
                move |result: Result<Vec<(String, String)>, String>| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::MonsterRef(
                            MonsterRefEditorMessage::MonsterNamesLoaded(result),
                        ),
                    )
                },
            )
        }
        MonsterRefEditorMessage::MonsterNamesLoaded(res) => {
            match res {
                Ok(names) => {
                    app.state.lookups.insert("monster_names".to_string(), names);
                }
                Err(e) => {
                    eprintln!("Failed to load monster names: {}", e);
                }
            }
            Task::none()
        }
        MonsterRefEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages_tab!(
                app,
                monster_ref_spreadsheets,
                monster_ref_editors,
                &tab_id,
                |index, field, value| crate::message::Message::monster_ref(
                    MonsterRefEditorMessage::FieldChanged(index, field, value)
                ),
                msg
            );
            Task::none()
        }
        MonsterRefEditorMessage::PaneResized(event) => {
            if let Some(ed) = app.state.monster_ref_editors.get_mut(&tab_id) {
                if let Some(ref mut ps) = ed.editor.pane_state {
                    ps.resize(event.split, event.ratio);
                }
            }
            if let Some(ss) = app.state.monster_ref_spreadsheets.get_mut(&tab_id) {
                if let Some(ref mut ps) = ss.pane_state {
                    ps.resize(event.split, event.ratio);
                }
            }
            Task::none()
        }
        MonsterRefEditorMessage::PaneClicked(pane) => {
            if let Some(ed) = app.state.monster_ref_editors.get_mut(&tab_id) {
                ed.editor.pane_focus = Some(pane);
            }
            Task::none()
        }
    }
}
