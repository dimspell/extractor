use std::path::PathBuf;

use dispel_core::NpcIni;
use iced::Task;

use crate::app::App;
use crate::message::{Message, MessageExt};
use crate::update::editor::tab::{self, lookup_keys};

mod component;

crate::define_tab_editor! {
    name: npc_ref,
    name_pascal: NpcRef,
    record: dispel_core::NPC,
    field: npc_ref_editor,
    loaded_prefix: "NPC ref catalog loaded: ",
    error_prefix: "Error loading NPC ref catalog: ",
    save_success_msg: "NPC refs saved successfully.",
    save_error_msg: "Error saving NPC refs",
    extension: {
        message: NpcRefExtension,
        handle: handle_extension,
        after_load: after_load,
    },
}

/// Editor-specific messages beyond the shared catalog-load flow.
#[derive(Debug, Clone)]
pub enum NpcRefExtension {
    NpcNamesLoaded(Result<Vec<(String, String)>, String>),
}

/// Kick off the NPC-name lookup after a catalog load if it is not cached.
fn after_load(app: &mut App) -> Task<Message> {
    if app.state.shared_game_path.is_empty()
        || app.state.lookups.contains_key(lookup_keys::NPC_NAMES)
    {
        return Task::none();
    }
    let path = PathBuf::from(&app.state.shared_game_path).join("Npc.ini");
    tab::load_name_lookup::<NpcIni, _, _>(
        path,
        |npcs| {
            npcs.iter()
                .map(|n| (n.id.to_string(), n.description.clone()))
                .collect()
        },
        |result| {
            Message::npc_ref(NpcRefEditorMessage::Extension(
                NpcRefExtension::NpcNamesLoaded(result),
            ))
        },
    )
}

fn handle_extension(msg: NpcRefExtension, app: &mut App, _tab_id: usize) -> Task<Message> {
    match msg {
        NpcRefExtension::NpcNamesLoaded(result) => {
            if let Ok(names) = result {
                app.state
                    .lookups
                    .insert(lookup_keys::NPC_NAMES.to_string(), names);
                // Same cache-refresh contract as monster_ref.
                tab::rebuild_all_caches(&mut app.state.editors.npc_ref_editor, &app.state.lookups);
            }
            Task::none()
        }
    }
}

mod view;
pub use view::view;
