use std::path::PathBuf;

use dispel_core::{Extractor, NpcIni, NPC};
use iced::Task;

use crate::app::App;
use crate::message::{Message, MessageExt};
use crate::update::editor::tab;

mod component;

crate::define_tab_editor! {
    name: npc_ref,
    name_pascal: NpcRef,
    record: NPC,
    field: npc_ref_editor,
    empty_text: "NPC ref file not loaded",
    save_success_msg: "NPC refs saved successfully.",
    save_error_msg: "Error saving NPC refs",
    extra_variants: {
        LoadCatalog(std::path::PathBuf),
        NpcNamesLoaded(Result<Vec<(String, String)>, String>),
    },
}

mod view;
pub use view::view;

pub fn handle(msg: NpcRefEditorMessage, app: &mut App) -> Task<Message> {
    let tab_id = tab::get_tab_id(&app.state.workspace);

    match msg {
        NpcRefEditorMessage::LoadCatalog(path) => {
            crate::components::item_catalog::ensure_item_lookups(
                &app.state.shared_game_path,
                &mut app.state.lookups,
            );
            tab::load_catalog_sync(
                path,
                &mut app.state.editors.npc_ref_editor,
                tab_id,
                &app.state.lookups,
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
                    move |result| Message::npc_ref(NpcRefEditorMessage::NpcNamesLoaded(result)),
                );
            }
            Task::none()
        }
        NpcRefEditorMessage::NpcNamesLoaded(result) => {
            if let Ok(names) = result {
                if app.state.editors.npc_ref_editor.contains_key(&tab_id) {
                    app.state.lookups.insert("NPC".to_string(), names);
                }
            }
            Task::none()
        }
        msg => handle_core(msg, app, tab_id),
    }
}
