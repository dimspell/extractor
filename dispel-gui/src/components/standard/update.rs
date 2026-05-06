use crate::components::editable::EditableRecord;
use crate::components::standard::message::StandardEditorMessage;
use crate::generic_editor::GenericEditorState;
use crate::loading_state::LoadingState;
use crate::message::Message;
use crate::view::editor::SpreadsheetState;
use dispel_core::Extractor;
use iced::Task;
use std::path::PathBuf;

/// Handle all standard arms of a `StandardEditorMessage<T>`.
///
/// The `Spreadsheet` variant is handled by the thin per-editor wrapper (it
/// must call the `handle_spreadsheet_messages!` macro with editor-specific
/// ident names). This function returns `Task::none()` if it ever receives
/// `Spreadsheet` — that arm should never reach here.
pub fn handle<T, F>(
    msg: StandardEditorMessage<T>,
    editor: &mut GenericEditorState<T>,
    spreadsheet: &mut SpreadsheetState,
    game_path: &str,
    file_path: &'static str,
    wrap: F,
) -> Task<Message>
where
    T: EditableRecord + Extractor + Clone + std::fmt::Debug + Send + 'static,
    F: Fn(StandardEditorMessage<T>) -> Message + Clone + Send + 'static,
{
    match msg {
        StandardEditorMessage::LoadCatalog => {
            if game_path.is_empty() {
                editor.status_msg = "Please select game path first.".into();
                return Task::none();
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
                    let wrap_caches = wrap.clone();
                    Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || {
                                crate::view::editor::compute_caches(&cat_for_caches)
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
            editor.update_field(idx, &field, value);
            Task::none()
        }

        StandardEditorMessage::Save => {
            if game_path.is_empty() {
                editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            if editor.catalog.is_some() {
                editor.status_msg = "Saving...".into();
                editor.loading_state = LoadingState::Loading;
                let result = editor.save(game_path, file_path);
                let wrap2 = wrap.clone();
                return Task::perform(async { result }, move |r| {
                    wrap2(StandardEditorMessage::Saved(r))
                });
            }
            editor.status_msg = "Nothing to save.".into();
            Task::none()
        }

        StandardEditorMessage::Saved(result) => {
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
    }
}
