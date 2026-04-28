use std::path::PathBuf;

use dispel_core::{Extractor, MonsterIni};
use iced::Task;

use super::tab;
use crate::app::App;
use crate::handle_spreadsheet_messages_tab;
use crate::message::editor::monster_ref::MonsterRefEditorMessage;
use crate::message::MessageExt;

pub fn handle(msg: MonsterRefEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = tab::get_tab_id(&app.state.workspace);

    match msg {
        MonsterRefEditorMessage::SelectEntry(index) => {
            tab::select(&mut app.state.monster_ref_editors, tab_id, index)
        }
        MonsterRefEditorMessage::FieldChanged(index, field, value) => tab::field_changed(
            &mut app.state.monster_ref_editors,
            tab_id,
            index,
            field,
            value,
        ),
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
