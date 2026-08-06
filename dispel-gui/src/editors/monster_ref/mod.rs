use std::path::PathBuf;

use dispel_core::{Extractor, MonsterIni, MonsterRef};
use iced::Task;

use crate::app::App;
use crate::message::{Message, MessageExt};
use crate::update::editor::tab;

mod component;

crate::define_tab_editor! {
    name: monster_ref,
    name_pascal: MonsterRef,
    record: MonsterRef,
    field: monster_ref_editor,
    empty_text: "Monster ref file not loaded",
    save_success_msg: "Monster ref saved successfully.",
    save_error_msg: "Error saving monster ref",
    extra_variants: {
        LoadCatalog(std::path::PathBuf),
        LoadMonsterNames,
        MonsterNamesLoaded(Result<Vec<(String, String)>, String>),
    },
}

mod view;
pub use view::view;

pub fn handle(msg: MonsterRefEditorMessage, app: &mut App) -> Task<Message> {
    let tab_id = tab::get_tab_id(&app.state.workspace);

    match msg {
        MonsterRefEditorMessage::LoadCatalog(path) => {
            crate::components::item_catalog::ensure_item_lookups(
                &app.state.shared_game_path,
                &mut app.state.lookups,
            );
            tab::load_catalog_sync(
                path,
                &mut app.state.editors.monster_ref_editor,
                tab_id,
                &app.state.lookups,
            );
            if !app.state.lookups.contains_key("monster_names") {
                return Task::done(Message::monster_ref(
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
                |result| Message::monster_ref(MonsterRefEditorMessage::MonsterNamesLoaded(result)),
            )
        }
        MonsterRefEditorMessage::MonsterNamesLoaded(result) => {
            match result {
                Ok(names) => {
                    app.state.lookups.insert("monster_names".to_string(), names);
                    // Rebuild display caches for all open tabs now that lookups
                    // are available, so the spreadsheet shows resolved names.
                    let tab_ids: Vec<usize> = app
                        .state
                        .editors
                        .monster_ref_editor
                        .spreadsheets
                        .keys()
                        .copied()
                        .collect();
                    for tab_id in tab_ids {
                        let ss = app
                            .state
                            .editors
                            .monster_ref_editor
                            .spreadsheets
                            .get_mut(&tab_id);
                        let ed = app.state.editors.monster_ref_editor.editors.get(&tab_id);
                        if let (Some(ss), Some(ed)) = (ss, ed)
                            && let Some(catalog) = ed.editor.catalog.as_ref()
                        {
                            ss.compute_all_caches(catalog, &app.state.lookups);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load monster names: {}", e);
                }
            }
            Task::none()
        }
        msg => handle_core(msg, app, tab_id),
    }
}
