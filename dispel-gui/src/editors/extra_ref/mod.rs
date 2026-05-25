use crate::app::App;
use crate::components::loading_state::LoadingState;
use crate::message::Message;
use crate::update::editor::tab;
use dispel_core::{ExtraRef, Extractor};
use iced::Task;

mod component;

crate::define_tab_editor! {
    name: extra_ref,
    name_pascal: ExtraRef,
    record: ExtraRef,
    field: extra_ref_editor,
    empty_text: "Extra ref file not loaded",
    save_success_msg: "Extra refs saved successfully.",
    save_error_msg: "Error saving extra refs",
    extra_variants: {
        /// tab_id captured at task-spawn time so the right editor is updated on async completion.
        CatalogLoaded(usize, Result<Vec<ExtraRef>, String>),
        LoadCatalog(std::path::PathBuf),
    },
}

mod view;
pub use view::view;

pub fn handle(msg: ExtraRefEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = tab::get_tab_id(&app.state.workspace);

    match msg {
        ExtraRefEditorMessage::LoadCatalog(path) => {
            crate::components::item_catalog::ensure_item_lookups(
                &app.state.shared_game_path,
                &mut app.state.lookups,
            );
            tab::load_catalog_sync(
                path.clone(),
                &mut app.state.extra_ref_editor,
                tab_id,
                &app.state.lookups,
            );
            Task::perform(
                async move {
                    <ExtraRef as Extractor>::read_file(&path)
                        .map_err(|e: std::io::Error| e.to_string())
                },
                move |result| {
                    Message::Editor(crate::message::editor::EditorMessage::ExtraRef(
                        ExtraRefEditorMessage::CatalogLoaded(tab_id, result),
                    ))
                },
            )
        }
        ExtraRefEditorMessage::CatalogLoaded(id, result) => {
            if let Some(editor) = app.state.extra_ref_editor.editors.get_mut(&id) {
                editor.editor.loading_state = LoadingState::Loaded(());
                match result {
                    Ok(catalog) => {
                        editor.editor.status_msg =
                            format!("Extra ref catalog loaded: {} entries", catalog.len());
                        editor.editor.catalog = Some(catalog.clone());
                        if let Some(ss) = app.state.extra_ref_editor.spreadsheets.get_mut(&id) {
                            ss.apply_filter(&catalog);
                            let lookups = &app.state.lookups;
                            ss.compute_all_caches(&catalog, lookups);
                            ss.init_pane_state();
                        }
                    }
                    Err(e) => {
                        editor.editor.status_msg =
                            format!("Error loading extra ref catalog: {}", e);
                    }
                }
            }
            Task::none()
        }
        msg => handle_core(msg, app, tab_id),
    }
}
