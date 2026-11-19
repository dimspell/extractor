use std::path::PathBuf;

use dispel_core::MonsterIni;
use iced::Task;

use crate::app::App;
use crate::message::{Message, MessageExt};
use crate::update::editor::tab::{self, lookup_keys};

mod component;

crate::define_tab_editor! {
    name: monster_ref,
    name_pascal: MonsterRef,
    record: dispel_core::MonsterRef,
    field: monster_ref_editor,
    loaded_prefix: "Monster ref catalog loaded: ",
    error_prefix: "Error loading monster ref catalog: ",
    save_success_msg: "Monster ref saved successfully.",
    save_error_msg: "Error saving monster ref",
    extension: {
        message: MonsterRefExtension,
        handle: handle_extension,
        after_load: after_load,
    },
}

/// Editor-specific messages beyond the shared catalog-load flow.
#[derive(Debug, Clone)]
pub enum MonsterRefExtension {
    LoadMonsterNames,
    MonsterNamesLoaded(Result<Vec<(String, String)>, String>),
}

/// Kick off the monster-name lookup after a catalog load if it is not cached.
fn after_load(app: &mut App) -> Task<Message> {
    if app.state.shared_game_path.is_empty()
        || app.state.lookups.contains_key(lookup_keys::MONSTER_NAMES)
    {
        return Task::none();
    }
    let path = PathBuf::from(&app.state.shared_game_path).join("Monster.ini");
    tab::load_name_lookup::<MonsterIni, _, _>(
        path,
        |monsters| {
            monsters
                .iter()
                .map(|m| (m.id.to_string(), m.name.clone().unwrap_or_default()))
                .collect()
        },
        |result| {
            Message::monster_ref(MonsterRefEditorMessage::Extension(
                MonsterRefExtension::MonsterNamesLoaded(result),
            ))
        },
    )
}

fn handle_extension(msg: MonsterRefExtension, app: &mut App, _tab_id: usize) -> Task<Message> {
    match msg {
        MonsterRefExtension::LoadMonsterNames => after_load(app),
        MonsterRefExtension::MonsterNamesLoaded(result) => {
            match result {
                Ok(names) => {
                    app.state
                        .lookups
                        .insert(lookup_keys::MONSTER_NAMES.to_string(), names);
                    // Names just became available — every open tab's display
                    // cache must be rebuilt so resolved names show up
                    // consistently.
                    tab::rebuild_all_caches(
                        &mut app.state.editors.monster_ref_editor,
                        &app.state.lookups,
                    );
                }
                Err(e) => eprintln!("Failed to load monster names: {}", e),
            }
            Task::none()
        }
    }
}

mod view;
pub use view::view;
