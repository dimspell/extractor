use std::path::PathBuf;

use dispel_core::{Extractor, MonsterIni};
use iced::Task;

use crate::app::App;
use crate::components::editable::EditableRecord;
use crate::editors::monster_ref::MonsterRefEditorMessage;
use crate::handle_spreadsheet_messages_tab;
use crate::message::MessageExt;
use crate::update::editor::tab;

pub fn handle(msg: MonsterRefEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = tab::get_tab_id(&app.state.workspace);

    match msg {
        MonsterRefEditorMessage::SelectEntry(index) => {
            tab::select(&mut app.state.monster_ref_editors, tab_id, index)
        }
        MonsterRefEditorMessage::FieldChanged(index, field, value) => {
            let (old_value, orig_idx, file_path) = app
                .state
                .monster_ref_editors
                .get(&tab_id)
                .map(|e| {
                    let (oidx, old) = e
                        .editor
                        .filtered
                        .get(index)
                        .map(|(i, r)| (*i as u32, r.get_field(&field)))
                        .unwrap_or((0, String::new()));
                    let fp = e.current_file.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
                    (old, oidx, fp)
                })
                .unwrap_or_default();
            let new_value = value.clone();
            let task = tab::field_changed(
                &mut app.state.monster_ref_editors,
                tab_id,
                index,
                field.clone(),
                value,
            );
            let observe = if old_value != new_value {
                crate::editors::mod_packager::recording::observe_field_change(
                    app, file_path, orig_idx, &field, old_value, new_value,
                )
            } else {
                iced::Task::none()
            };
            observe.chain(task)
        }
        MonsterRefEditorMessage::Save => tab::save(
            &mut app.state.monster_ref_editors,
            tab_id,
            "Monster ref saved successfully.",
            "Error saving monster ref",
        ),
        MonsterRefEditorMessage::AddEntry => {
            tab::add_entry(&mut app.state.monster_ref_editors, tab_id)
        }
        MonsterRefEditorMessage::RemoveEntry(index) => {
            tab::remove_entry(&mut app.state.monster_ref_editors, tab_id, index)
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
        MonsterRefEditorMessage::PaneResized(event) => tab::pane_resized(
            &mut app.state.monster_ref_editors,
            &mut app.state.monster_ref_spreadsheets,
            tab_id,
            event,
        ),
        MonsterRefEditorMessage::PaneClicked(pane) => {
            tab::pane_clicked(&mut app.state.monster_ref_editors, tab_id, pane)
        }
        MonsterRefEditorMessage::LoadCatalog(path) => {
            tab::load_catalog_sync(
                path,
                &mut app.state.monster_ref_editors,
                &mut app.state.monster_ref_spreadsheets,
                tab_id,
            );
            if !app.state.lookups.contains_key("monster_names") {
                return Task::done(crate::message::Message::monster_ref(
                    MonsterRefEditorMessage::LoadMonsterNames,
                ));
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
                |result: Result<Vec<(String, String)>, String>| {
                    crate::message::Message::monster_ref(
                        MonsterRefEditorMessage::MonsterNamesLoaded(result),
                    )
                },
            )
        }
        MonsterRefEditorMessage::MonsterNamesLoaded(result) => {
            match result {
                Ok(names) => {
                    app.state.lookups.insert("monster_names".to_string(), names);
                }
                Err(e) => {
                    eprintln!("Failed to load monster names: {}", e);
                }
            }
            Task::none()
        }
    }
}
