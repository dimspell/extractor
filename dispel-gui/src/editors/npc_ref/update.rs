use std::path::PathBuf;

use dispel_core::{Extractor, NpcIni};
use iced::Task;

use super::tab;
use crate::app::App;
use crate::handle_spreadsheet_messages_tab;
use crate::message::editor::npc_ref::NpcRefEditorMessage;
use crate::message::MessageExt;

pub fn handle(msg: NpcRefEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = tab::get_tab_id(&app.state.workspace);

    match msg {
        NpcRefEditorMessage::Select(index) => {
            tab::select(&mut app.state.npc_ref_editors, tab_id, index)
        }
        NpcRefEditorMessage::FieldChanged(index, field, value) => {
            tab::field_changed(&mut app.state.npc_ref_editors, tab_id, index, field, value)
        }
        NpcRefEditorMessage::Save => tab::save(
            &mut app.state.npc_ref_editors,
            tab_id,
            "NPC refs saved successfully.",
            "Error saving NPC refs",
        ),
        NpcRefEditorMessage::AddEntry => tab::add_entry(&mut app.state.npc_ref_editors, tab_id),
        NpcRefEditorMessage::RemoveEntry(index) => {
            tab::remove_entry(&mut app.state.npc_ref_editors, tab_id, index)
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
        NpcRefEditorMessage::PaneResized(event) => tab::pane_resized(
            &mut app.state.npc_ref_editors,
            &mut app.state.npc_ref_spreadsheets,
            tab_id,
            event,
        ),
        NpcRefEditorMessage::PaneClicked(pane) => {
            tab::pane_clicked(&mut app.state.npc_ref_editors, tab_id, pane)
        }
        NpcRefEditorMessage::LoadCatalog(path) => {
            tab::load_catalog_sync(
                path,
                &mut app.state.npc_ref_editors,
                &mut app.state.npc_ref_spreadsheets,
                tab_id,
            );
            if !app.state.lookups.contains_key("NPC") {
                let game_path = app.state.shared_game_path.clone();
                return Task::perform(
                    async move {
                        NpcIni::read_file(&PathBuf::from(&game_path).join("Npc.ini"))
                            .map(|npcs| {
                                npcs.iter()
                                    .map(|n| (n.id.to_string(), n.description.clone()))
                                    .collect()
                            })
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    |result| {
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
                // Only store the lookup if the tab is still open; discard stale async results.
                if app.state.npc_ref_editors.contains_key(&tab_id) {
                    app.state.lookups.insert("NPC".to_string(), names);
                }
            }
            Task::none()
        }
    }
}
