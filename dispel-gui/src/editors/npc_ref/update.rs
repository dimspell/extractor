use std::path::PathBuf;

use dispel_core::{Extractor, NpcIni};
use iced::Task;

use crate::app::App;
use crate::editors::mod_packager::recording::{
    capture_field_recording_context, observe_field_change,
};
use crate::editors::npc_ref::NpcRefEditorMessage;
use crate::handle_spreadsheet_messages_tab;
use crate::message::MessageExt;
use crate::update::editor::tab;

pub fn handle(msg: NpcRefEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = tab::get_tab_id(&app.state.workspace);

    match msg {
        NpcRefEditorMessage::Select(index) => {
            tab::select(&mut app.state.npc_ref_editors, tab_id, index)
        }
        NpcRefEditorMessage::FieldChanged(index, field, value) => {
            let captured = capture_field_recording_context(
                app.state.npc_ref_editors.get(&tab_id),
                index,
                &field,
                &app.state.shared_game_path,
            );
            let new_value = value.clone();
            let task = tab::field_changed(
                &mut app.state.npc_ref_editors,
                tab_id,
                index,
                field.clone(),
                value,
            );
            match captured {
                Some((old_value, orig_idx, file_path)) if old_value != new_value => {
                    let observe = observe_field_change(
                        app, file_path, orig_idx, &field, old_value, new_value,
                    );
                    task.chain(observe)
                }
                _ => task,
            }
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
