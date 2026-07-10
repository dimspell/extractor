use iced::Task;

use crate::app::App;
use crate::editors::save_file_viewer::message::SaveFileViewerMessage;
use crate::message::{Message, MessageExt};

pub fn handle(msg: SaveFileViewerMessage, app: &mut App) -> Task<Message> {
    let tab_id = match app.state.workspace.active() {
        Some(t) => t.id,
        None => return Task::none(),
    };

    let state = match app.state.editors.save_file_viewers.get_mut(&tab_id) {
        Some(s) => s,
        None => return Task::none(),
    };

    match msg {
        SaveFileViewerMessage::SelectSection(section) => {
            state.active_section = section;
            Task::none()
        }
        SaveFileViewerMessage::SelectCategory(cat) => {
            state.inventory_category = Some(cat);
            Task::none()
        }
        SaveFileViewerMessage::HexViewer(index, msg) => {
            if let Some(viewer) = state.raw_hex_viewers.get_mut(index) {
                hexedit::update(&mut viewer.state, &hexedit::HexEditorConfig::default(), msg)
                    .map(Message::hex_editor)
            } else {
                Task::none()
            }
        }
        SaveFileViewerMessage::InventoryHexViewer(cat, msg) => {
            if let Some(viewer) = state.inventory_hex_viewers.get_mut(&cat) {
                hexedit::update(viewer, &hexedit::HexEditorConfig::default(), msg)
                    .map(Message::hex_editor)
            } else {
                Task::none()
            }
        }
        SaveFileViewerMessage::SelectJournalSection(section) => {
            state.journal_section = section;
            state.selected_journal_entry = None;
            Task::none()
        }
        SaveFileViewerMessage::SelectMap(index) => {
            state.selected_map = Some(index);
            Task::none()
        }
        SaveFileViewerMessage::Load(_) => {
            // Load is handled by app.rs::open_file_in_workspace via Task::perform
            state.loading = true;
            Task::none()
        }
        SaveFileViewerMessage::Loaded(result) => {
            state.loading = false;
            match result {
                Ok(loaded) => {
                    state.save_file = Some(loaded.save_file.clone());
                    // Build events display cache
                    let n = loaded.save_file.events.len();
                    let mut display_cache = Vec::with_capacity(n);
                    for (i, ev) in loaded.save_file.events.iter().enumerate() {
                        display_cache.push(vec![
                            format!("{}", i + 1),
                            format!("{} : {}", ev.unknown_1, ev.unknown_2),
                            ev.script_name.clone(),
                        ]);
                    }
                    state.events_display_cache = display_cache;
                    state.events_filtered_indices = (0..n).collect();
                    state.raw_hex_viewers = loaded
                        .hex_editors
                        .into_iter()
                        .map(|d| {
                            use crate::editors::save_file_viewer::state::RawHexViewer;
                            let editor = hexedit::HexEditorState::from_bytes(
                                d.label,
                                d.data.clone(),
                                None,
                                None,
                            );
                            RawHexViewer {
                                label: d.label,
                                state: editor,
                            }
                        })
                        .collect();
                    // Build inventory hex viewers
                    // let inv = &loaded.save_file.inventory;
                    // for (cat, data) in [
                    //     (InventoryCategory::Event, inv.event_items.clone()),
                    //     (InventoryCategory::Misc, inv.misc_items.clone()),
                    //     (InventoryCategory::Edit, inv.edit_items.clone()),
                    //     (InventoryCategory::Weapon, inv.weapon_items.clone()),
                    //     (InventoryCategory::Heal, inv.heal_items.clone()),
                    // ] {
                    //     let label = format!("{} ({} bytes)", cat.label(), data.len());
                    //     let editor = hexedit::HexEditorState::from_bytes(
                    //         &label, data, None, None,
                    //     );
                    //     state.inventory_hex_viewers.insert(cat, editor);
                    // }
                    state.error = None;
                }
                Err(e) => {
                    state.error = Some(e);
                }
            }
            Task::none()
        }
    }
}
