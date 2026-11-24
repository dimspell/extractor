use std::collections::HashMap;

use crate::components::editable::EditableRecord;
use crate::components::loading_state::LoadingState;
use crate::components::standard::message::StandardEditorMessage;
use crate::components::standard::state::StandardEditor;
use crate::message::Message;
use dispel_core::Extractor;
use iced::Task;
use std::path::PathBuf;

/// Handle all standard arms of a `StandardEditorMessage<T>`.
///
/// The `Spreadsheet` variant is handled by the thin per-editor wrapper, which
/// calls the `handle_spreadsheet_messages!` macro with editor-specific ident
/// names. If `Spreadsheet` reaches this function, its match arm is a no-op that
/// returns `Task::none()`.
///
/// Returns the produced task along with a flag telling whether a `FieldChanged`
/// message actually applied a valid edit. The caller uses that flag to decide
/// whether to mark the tab dirty and record the edit.
pub fn handle<T, F>(
    msg: StandardEditorMessage<T>,
    bundle: &mut StandardEditor<T>,
    game_path: &str,
    lookups: &HashMap<String, Vec<(String, String)>>,
    file_path: &'static str,
    tab_id: usize,
    wrap: F,
) -> (Task<Message>, bool)
where
    T: EditableRecord + Extractor + Clone + std::fmt::Debug + Send + 'static,
    F: Fn(StandardEditorMessage<T>) -> Message + Clone + Send + 'static,
{
    let editor = &mut bundle.state;
    let spreadsheet = &mut bundle.spreadsheet;
    let task = match msg {
        StandardEditorMessage::LoadCatalog => {
            if game_path.is_empty() {
                editor.status_msg = "Please select game path first.".into();
                return (Task::none(), false);
            }
            editor.loading_state = LoadingState::Loading;
            spreadsheet.is_loading = true;
            let path = PathBuf::from(game_path).join(file_path);
            Task::perform(
                async move { T::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                move |result| wrap(StandardEditorMessage::CatalogLoaded(result)),
            )
        }

        StandardEditorMessage::CatalogLoaded(res) => {
            editor.loading_state = LoadingState::Loaded(());
            match res {
                Ok(catalog) => {
                    editor.status_msg = format!("Loaded {} records.", catalog.len());
                    editor.catalog = Some(catalog.clone());
                    editor.refresh();
                    editor.init_pane_state();
                    spreadsheet.apply_filter(&catalog);
                    // Cache build can take 100s of ms on a 10k-row catalog —
                    // run it in a blocking-pool worker and route the result
                    // back through `SpreadsheetMessage::CachesComputed`. The
                    // spreadsheet keeps `is_loading = true` until the result
                    // arrives so the UI shows the progress indicator.
                    let cat_for_caches = catalog;
                    let lookups_for_caches = lookups.clone();
                    let wrap_caches = wrap.clone();
                    Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || {
                                let mut data = crate::view::editor::compute_caches(&cat_for_caches);
                                crate::view::editor::spreadsheet::caches::
                                    resolve_composite_displays::<T>(
                                        &mut data,
                                        &lookups_for_caches,
                                    );
                                crate::view::editor::spreadsheet::caches::resolve_lookup_displays::<
                                    T,
                                >(&mut data, &lookups_for_caches);
                                data
                            })
                            .await
                            .unwrap_or_default()
                        },
                        move |data| {
                            wrap_caches(StandardEditorMessage::Spreadsheet(
                                crate::view::editor::SpreadsheetMessage::CachesComputed(data),
                            ))
                        },
                    )
                }
                Err(e) => {
                    editor.status_msg = format!("Error loading: {}", e);
                    spreadsheet.is_loading = false;
                    Task::none()
                }
            }
        }

        StandardEditorMessage::Select(idx) => {
            editor.select(idx);
            Task::none()
        }

        StandardEditorMessage::FieldChanged(idx, field, value) => {
            return (Task::none(), editor.update_field(idx, &field, value));
        }

        StandardEditorMessage::Save => {
            if game_path.is_empty() {
                editor.status_msg = "Please select game path first.".into();
                return (Task::none(), false);
            }
            if editor.catalog.is_some() {
                editor.status_msg = "Saving...".into();
                editor.loading_state = LoadingState::Loading;
                let result = editor.save(game_path, file_path);
                let wrap2 = wrap.clone();
                return (
                    Task::perform(async { result }, move |r| {
                        wrap2(StandardEditorMessage::Saved(tab_id, r))
                    }),
                    false,
                );
            }
            editor.status_msg = "Nothing to save.".into();
            Task::none()
        }

        StandardEditorMessage::Saved(_tab_id, result) => {
            editor.loading_state = LoadingState::Loaded(());
            match result {
                Ok(()) => editor.status_msg = "Saved successfully.".into(),
                Err(e) => editor.status_msg = format!("Error saving: {}", e),
            }
            Task::none()
        }

        StandardEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = editor.pane_state {
                ps.resize(event.split, event.ratio);
            }
            if let Some(ref mut ps) = spreadsheet.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }

        StandardEditorMessage::PaneClicked(pane) => {
            editor.pane_focus = Some(pane);
            Task::none()
        }

        StandardEditorMessage::Spreadsheet(_) => {
            // Handled by the thin per-editor wrapper via handle_spreadsheet_messages!
            Task::none()
        }
    };
    (task, false)
}
