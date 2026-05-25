//! Common Editor Handler Framework
//!
//! Provides macros and helper functions to handle common editor operations
//! with good user feedback and clear error handling.

// ===========================================================================
// Consolidated spreadsheet handler — single implementation, two entry points
// ===========================================================================
//
// The two public macros (`handle_spreadsheet_messages!` and
// `handle_spreadsheet_messages_tab!`) are ~95% identical.  Rather than
// duplicating the entire match body, they delegate to an internal macro
// `handle_spreadsheet_messages_inner!` which takes all access paths as
// parameters.
//
// Differences between the two callers:
//
// | Aspect            | Non-tab                     | Tab-editor                         |
// |-------------------|-----------------------------|------------------------------------|
// | ss (Spreadsheet)  | `$app.state.$field.spreadsheet`     | `ss` (local binding from `get_mut`)       |
// | catalog (Option)  | `&$app.state.$field.state.catalog` | `&ed.editor.catalog`                 |
// | make_inspector    | `$app.state.$field.make_inspector_textarea_contents(orig_idx)` | `ed.editor.make_inspector_textarea_contents(orig_idx)` |
// | unique_values     | `$app.state.$field.unique_values_for_column(col)` | `ed.editor.unique_values_for_column(col)` |
// | status_msg        | `&mut $app.state.$field.status_msg` | `&mut ed.editor.status_msg`          |
// | toggle_inspector  | populates textarea_contents  | (none)                              |
// | compute_caches    | direct field access          | re-borrow from hashmap (NLL-safe)   |

// ──────────────────────────────────────────────
// Internal implementation — all differences parameterized
// ──────────────────────────────────────────────
#[macro_export]
#[doc(hidden)]
macro_rules! handle_spreadsheet_messages_inner {
    (
        app: $app:ident,
        ss: $ss:expr,
        catalog: $catalog_provider:expr,
        make_inspector: $make_inspector:expr,
        unique_values: $unique_values:expr,
        status_msg: $status_msg:expr,
        compute_caches: $compute_caches:block,
        toggle_inspector_extra: $toggle_inspector_extra:block,
        field_changed_msg: $field_changed_msg:expr,
        msg: $msg:ident,
    ) => {
        use $crate::view::editor::SpreadsheetMessage as SM;
        match $msg {
            SM::ToggleActive => {
                $ss.toggle_active();
                if $ss.active {
                    if let Some(catalog) = $catalog_provider() {
                        $ss.init_filter(catalog);
                        let lookups = &$app.state.lookups;
                        $ss.compute_all_caches(catalog, lookups);
                        $ss.init_pane_state();
                    }
                }
            }
            SM::SortColumn(col) => {
                $ss.toggle_sort(col);
                if let Some(catalog) = $catalog_provider() {
                    $ss.apply_sort(catalog);
                }
            }
            SM::FilterChanged(query) => {
                $ss.filter_query = query;
                if let Some(catalog) = $catalog_provider() {
                    $ss.apply_filter(catalog);
                    $ss.record_target_offset(0.0, 0.0);
                }
            }
            SM::ClearFilter => {
                if let Some(catalog) = $catalog_provider() {
                    $ss.clear_filter(catalog);
                }
            }
            SM::SetFilterMode(mode) => {
                if let Some(catalog) = $catalog_provider() {
                    $ss.set_filter_mode(mode, catalog);
                }
            }
            SM::NavigateNextHighlight => {
                $ss.navigate_next_highlight();
                if let Some(orig_idx) = $ss.current_highlight_orig_idx() {
                    if let Some(fidx) = $ss.filtered_indices.iter().position(|&i| i == orig_idx) {
                        $ss.set_selection(fidx);
                        let y = $ss.scroll_y_for_row(fidx);
                        let x = $ss.horizontal_scroll_offset;
                        $ss.record_target_offset(x, y);
                    }
                }
            }
            SM::NavigatePrevHighlight => {
                $ss.navigate_prev_highlight();
                if let Some(orig_idx) = $ss.current_highlight_orig_idx() {
                    if let Some(fidx) = $ss.filtered_indices.iter().position(|&i| i == orig_idx) {
                        $ss.set_selection(fidx);
                        let y = $ss.scroll_y_for_row(fidx);
                        let x = $ss.horizontal_scroll_offset;
                        $ss.record_target_offset(x, y);
                    }
                }
            }
            SM::NavigateUp => {
                if let Some(fidx) = $ss.navigate_up() {
                    if let Some(&orig_idx) = $ss.filtered_indices.get(fidx) {
                        $ss.inspector_textarea_contents = $make_inspector(orig_idx);
                    }
                    let y = $ss.ensure_row_visible_y(fidx);
                    let x = $ss.horizontal_scroll_offset;
                    $ss.record_target_offset(x, y);
                }
            }
            SM::NavigateDown => {
                if let Some(fidx) = $ss.navigate_down() {
                    if let Some(&orig_idx) = $ss.filtered_indices.get(fidx) {
                        $ss.inspector_textarea_contents = $make_inspector(orig_idx);
                    }
                    let y = $ss.ensure_row_visible_y(fidx);
                    let x = $ss.horizontal_scroll_offset;
                    $ss.record_target_offset(x, y);
                }
            }
            SM::NavigateTop => {
                if let Some(fidx) = $ss.navigate_top() {
                    if let Some(&orig_idx) = $ss.filtered_indices.get(fidx) {
                        $ss.inspector_textarea_contents = $make_inspector(orig_idx);
                    }
                    let x = $ss.horizontal_scroll_offset;
                    $ss.record_target_offset(x, 0.0);
                }
            }
            SM::NavigateBottom => {
                if let Some(fidx) = $ss.navigate_bottom() {
                    if let Some(&orig_idx) = $ss.filtered_indices.get(fidx) {
                        $ss.inspector_textarea_contents = $make_inspector(orig_idx);
                    }
                    let y = $ss.scroll_y_for_row(fidx);
                    let x = $ss.horizontal_scroll_offset;
                    $ss.record_target_offset(x, y);
                }
            }
            SM::SelectRow(filtered_idx) => {
                $ss.select_row(filtered_idx);
                $ss.ensure_inspector_pane();
                if let Some(&orig_idx) = $ss.filtered_indices.get(filtered_idx) {
                    $ss.inspector_textarea_contents = $make_inspector(orig_idx);
                } else {
                    $ss.inspector_textarea_contents.clear();
                }
            }
            SM::TextAreaChanged(orig_idx, field, action) => {
                if let Some(tc) = $ss.inspector_textarea_contents.get_mut(&field) {
                    tc.0.perform(action);
                    let raw = tc.0.text();
                    let new_text = raw.strip_suffix('\n').unwrap_or(&raw).to_string();
                    let msg = $field_changed_msg(orig_idx, field, new_text);
                    let task = $app.update(msg);
                    $compute_caches
                    return task;
                }
            }
            SM::InspectorFieldChanged(orig_idx, field, value) => {
                let msg = $field_changed_msg(orig_idx, field, value);
                let task = $app.update(msg);
                $compute_caches
                return task;
            }
            SM::CachesComputed(data) => {
                $ss.install_caches(data);
                $ss.is_loading = false;
            }
            SM::CancelEdit => {
                if $ss.resizing_column.is_some() {
                    $ss.end_column_resize();
                }
            }
            SM::ToggleInspector => {
                $ss.toggle_inspector();
                $ss.ensure_inspector_pane();
                $toggle_inspector_extra
            }
            SM::CloseInspector => {
                $ss.show_inspector = false;
                $ss.ensure_inspector_pane();
            }
            SM::ExportCsv => {
                if let Some(catalog) = $catalog_provider() {
                    match $ss.to_csv_bytes(catalog) {
                        Ok(bytes) => {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_file_name("export.csv")
                                .add_filter("CSV", &["csv"])
                                .save_file()
                            {
                                match std::fs::write(&path, &bytes) {
                                    Ok(_) => {
                                        *$status_msg =
                                            format!("Exported CSV to {}", path.display());
                                    }
                                    Err(e) => {
                                        *$status_msg =
                                            format!("CSV export failed: {}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            *$status_msg = format!("CSV export failed: {}", e);
                        }
                    }
                }
            }
            SM::CsvExported(result) => match result {
                Ok(path) => {
                    *$status_msg = format!("Exported CSV to {}", path.display());
                }
                Err(e) if e == "cancelled" => {}
                Err(e) => {
                    *$status_msg = format!("CSV export failed: {}", e);
                }
            },
            SM::BodyScrolled(offset, viewport_height) => {
                $ss.record_scroll(offset.x, offset.y, viewport_height);
            }
            SM::StartResizeColumn(col) => {
                if $ss.try_begin_column_resize(col) {
                    if let Some(catalog) = $catalog_provider() {
                        $ss.auto_size_column(col, catalog);
                    }
                }
            }
            SM::ResizeColumnCursor(x) => $ss.update_column_resize(x),
            SM::EndResizeColumn => $ss.end_column_resize(),
            SM::ResetColumnWidth(col) => {
                if let Some(catalog) = $catalog_provider() {
                    $ss.auto_size_column(col, catalog);
                }
            }
            SM::OpenColumnFilter(col) => {
                if $ss.active_column_filter == Some(col) {
                    $ss.active_column_filter = None;
                    $ss.column_filter_search.clear();
                } else {
                    $ss.column_filter_options = $unique_values(col);
                    $ss.active_column_filter = Some(col);
                    $ss.column_filter_search.clear();
                }
            }
            SM::CloseColumnFilterModal => {
                $ss.active_column_filter = None;
                $ss.column_filter_search.clear();
            }
            SM::ApplyColumnFilter(col, value) => {
                let mut set = std::collections::HashSet::new();
                set.insert(value);
                $ss.column_filters.insert(col, set);
                $ss.active_column_filter = None;
                if let Some(catalog) = $catalog_provider() {
                    $ss.apply_filter(catalog);
                    $ss.apply_sort(catalog);
                    $ss.record_target_offset(0.0, 0.0);
                }
            }
            SM::ClearColumnFilter(col) => {
                if let Some(catalog) = $catalog_provider() {
                    $ss.clear_column_filter(col, catalog);
                    $ss.apply_sort(catalog);
                    $ss.record_target_offset(0.0, 0.0);
                }
            }
            SM::QuickFilter(col, value) => {
                let mut set = std::collections::HashSet::new();
                set.insert(value);
                $ss.column_filters.insert(col, set);
                if let Some(catalog) = $catalog_provider() {
                    $ss.apply_filter(catalog);
                    $ss.apply_sort(catalog);
                    $ss.record_target_offset(0.0, 0.0);
                }
            }
            SM::ColumnFilterSearch(query) => {
                $ss.column_filter_search = query;
            }
            SM::ToggleColumnFilterValue(col, value) => {
                let entry = $ss.column_filters.entry(col).or_default();
                if entry.contains(&value) {
                    entry.remove(&value);
                } else {
                    entry.insert(value);
                }
                if let Some(catalog) = $catalog_provider() {
                    $ss.apply_filter(catalog);
                    $ss.apply_sort(catalog);
                    $ss.record_target_offset(0.0, 0.0);
                }
            }
            SM::SelectAllColumnFilter(col) => {
                let all_values: std::collections::HashSet<String> = $ss
                    .column_filter_options
                    .iter()
                    .map(|opt| opt.value.clone())
                    .collect();
                $ss.column_filters.insert(col, all_values);
                if let Some(catalog) = $catalog_provider() {
                    $ss.apply_filter(catalog);
                    $ss.apply_sort(catalog);
                    $ss.record_target_offset(0.0, 0.0);
                }
            }
            SM::ClearAllColumnFilter(col) => {
                $ss.column_filters.remove(&col);
                $ss.column_filter_search.clear();
                if let Some(catalog) = $catalog_provider() {
                    $ss.apply_filter(catalog);
                    $ss.apply_sort(catalog);
                    $ss.record_target_offset(0.0, 0.0);
                }
            }
        }
    };
}

// ──────────────────────────────────────────────
// Thin wrapper for single-file editors (StandardEditor<T>)
// ──────────────────────────────────────────────
/// Macro to handle spreadsheet messages for single-file editors.
///
/// The `$field` ident is the field on `AppState` (e.g. `weapon_editor`, `wave_ini_editor`).
/// Delegates to [`handle_spreadsheet_messages_inner!`].
#[macro_export]
macro_rules! handle_spreadsheet_messages {
    ($app:ident, $field:ident, $field_changed_msg:expr, $msg:ident) => {
        $crate::handle_spreadsheet_messages_inner! {
            app: $app,
            ss: $app.state.$field.spreadsheet,
            catalog: || &$app.state.$field.state.catalog,
            make_inspector: |idx| $app.state.$field.make_inspector_textarea_contents(idx),
            unique_values: |col| $app.state.$field.unique_values_for_column(col),
            status_msg: &mut $app.state.$field.status_msg,
            compute_caches: {
                if let Some(c) = &$app.state.$field.state.catalog {
                    let c = c.clone();
                    let lookups = &$app.state.lookups;
                    $app.state.$field.spreadsheet.compute_all_caches(&c, lookups);
                }
            },
            toggle_inspector_extra: {
                if $app.state.$field.spreadsheet.show_inspector {
                    if let Some(orig_idx) = $app.state.$field.spreadsheet.selected_orig {
                        $app.state.$field.spreadsheet.inspector_textarea_contents =
                            $app.state.$field.make_inspector_textarea_contents(orig_idx);
                    }
                }
            },
            field_changed_msg: $field_changed_msg,
            msg: $msg,
        }
    };
}

// ──────────────────────────────────────────────
// Thin wrapper for tab-based editors (TabbedEditor<T>)
// ──────────────────────────────────────────────
/// Macro to handle spreadsheet messages for tab-based editors (NpcRef, MonsterRef, etc.).
///
/// The `$tabbed_editor` ident is a field on `AppState` of type [`TabbedEditor<T>`].
/// Delegates to [`handle_spreadsheet_messages_inner!`].
#[macro_export]
macro_rules! handle_spreadsheet_messages_tab {
    ($app:ident, $tabbed_editor:ident, $tab_id:expr, $field_changed_msg:expr, $msg:ident) => {
        match $msg {
            other => {
                if let (Some(ed), Some(ss)) = (
                    $app.state.$tabbed_editor.editors.get_mut(&$tab_id),
                    $app.state.$tabbed_editor.spreadsheets.get_mut(&$tab_id),
                ) {
                    $crate::handle_spreadsheet_messages_inner! {
                        app: $app,
                        ss: ss,
                        catalog: || &ed.editor.catalog,
                        make_inspector: |idx| ed.editor.make_inspector_textarea_contents(idx),
                        unique_values: |col| ed.editor.unique_values_for_column(col),
                        status_msg: &mut ed.editor.status_msg,
                        compute_caches: {
                            let ss2 = $app.state.$tabbed_editor.spreadsheets.get_mut(&$tab_id);
                            let ed2 = $app.state.$tabbed_editor.editors.get_mut(&$tab_id);
                            if let (Some(ss2), Some(ed2)) = (ss2, ed2) {
                                if let Some(c) = &ed2.editor.catalog {
                                    let c = c.clone();
                                    let lookups = &$app.state.lookups;
                                    ss2.compute_all_caches(&c, lookups);
                                }
                            }
                        },
                        toggle_inspector_extra: {},
                        field_changed_msg: $field_changed_msg,
                        msg: other,
                    }
                }
            }
        }
    };
}

/// Macro to handle load catalog messages with async file reading
#[macro_export]
macro_rules! handle_load_catalog {
    ($app:ident, $editor:ident, $item_name:expr, $db_path:expr, $extractor:ty, $loaded_variant:expr) => {
        {
            if $app.state.shared_game_path.is_empty() {
                $app.state.$field.status_msg = "Please select game path first.".into();
                return Task::none();
            }

            $app.state.$field.loading_state = $crate::components::loading_state::LoadingState::Loading;
            $app.state.$field.status_msg = concat!($item_name, " catalog...").into();

            let path = std::path::PathBuf::from(&$app.state.shared_game_path).join($db_path);

            Task::perform(
                async move { <$extractor>::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                move |result: Result<Vec<$extractor>, String>| {
                    $loaded_variant(result)
                },
            )
        }
    };
}

/// Macro to handle catalog loaded messages
#[macro_export]
macro_rules! handle_catalog_loaded {
    ($app:ident, $editor:ident, $item_name:expr, $result:expr) => {{
        $app.state.$field.loading_state =
            $crate::components::loading_state::LoadingState::Loaded(());
        match $result {
            Ok(catalog) => {
                $app.state.$field.catalog = Some(catalog.clone());
                $app.state.$field.status_msg = format!(
                    concat!($item_name, " catalog loaded: {} {}"),
                    catalog.len(),
                    $item_name
                );
                $app.state.$field.refresh();
                $app.state.$field.init_pane_state();
                Task::none()
            }
            Err(e) => {
                let msg = format!(concat!("Failed to load ", $item_name, ": {}"), e);
                $app.state.$field.status_msg = msg.clone();
                Task::done($crate::message::Message::System(
                    $crate::message::SystemMessage::ShowError(msg),
                ))
            }
        }
    }};
}

/// Macro to handle select item messages
#[macro_export]
macro_rules! handle_select_item {
    ($app:ident, $editor:ident, $index:expr) => {{
        $app.state.$editor.select($index);
        Task::none()
    }};
}

/// Macro to handle field changed messages
#[macro_export]
macro_rules! handle_field_changed {
    ($app:ident, $editor:ident, $index:expr, $field:expr, $value:expr) => {{
        $app.state.$field.update_field($index, &$field, $value);
        Task::none()
    }};
}

/// Macro to handle save messages
#[macro_export]
macro_rules! handle_save {
    ($app:ident, $editor:ident, $item_name:expr, $save_method:expr, $saved_variant:expr) => {{
        if $app.state.shared_game_path.is_empty() {
            $app.state.$field.status_msg = "Please select game path first.".into();
            return Task::none();
        }

        $app.state.$field.loading_state = $crate::components::loading_state::LoadingState::Loading;
        let result = $save_method;

        Task::perform(async { result }, move |result: Result<(), String>| {
            $saved_variant(result)
        })
    }};
}

/// Macro to handle pane resize messages
#[macro_export]
macro_rules! handle_pane_resized {
    ($app:ident, $editor:ident, $event:ident) => {{
        if let Some(ref mut ps) = $app.state.$field.pane_state {
            ps.resize($event.split, $event.ratio);
        }
        Task::none()
    }};
}

/// Macro to handle pane clicked messages
#[macro_export]
macro_rules! handle_pane_clicked {
    ($app:ident, $editor:ident, $pane:ident) => {{
        $app.state.$field.pane_focus = Some($pane);
        Task::none()
    }};
}

/// Macro to handle saved messages
#[macro_export]
macro_rules! handle_saved {
    ($app:ident, $editor:ident, $item_name:expr, $result:expr) => {{
        $app.state.$field.loading_state =
            $crate::components::loading_state::LoadingState::Loaded(());
        match $result {
            Ok(_) => {
                $app.state.$field.status_msg = format!(concat!($item_name, " saved successfully"));
            }
            Err(e) => {
                $app.state.$field.status_msg =
                    format!(concat!("Error saving ", $item_name, ": {}"), e);
            }
        }
        Task::none()
    }};
}

/// Macro to create a simple message handler that just returns Task::none()
#[macro_export]
macro_rules! handle_simple {
    ($($msg_pattern:pat => $($body:tt)*),*) => {
        {
            match message {
                $($msg_pattern => {
                    $($body)*
                    Task::none()
                })*
            }
        }
    };
}

/// Macro to handle todo messages
#[macro_export]
macro_rules! handle_todo {
    () => {
        Task::none() // TODO: Implement this handler
    };
}

/// Macro to handle unsupported messages
#[macro_export]
macro_rules! handle_unsupported {
    ($item_name:expr) => {{
        eprintln!("Unsupported message for {}: {:?}", $item_name, message);
        Task::none()
    }};
}

/// Macro to handle error messages with logging
#[macro_export]
macro_rules! handle_error {
    ($app:ident, $editor:ident, $error_msg:expr) => {{
        $app.state.$field.status_msg = $error_msg.into();
        Task::none()
    }};
}

//     }};
// }

// Helper function to format errors
// pub fn format_errors(errors: Vec<(usize, String)>, item_name: &str) -> String {
//     if errors.len() > 5 {
//         let summary: Vec<_> = errors
//             .iter()
//             .take(5)
//             .map(|(idx, e)| format!("#{}: {}", idx, e))
//             .collect();
//         format!(
//             "Found {} errors in {}:\n{}\n... and {} more",
//             errors.len(),
//             item_name,
//             summary.join("\n"),
//             errors.len() - 5
//         )
//     } else {
//         let summary: Vec<_> = errors
//             .iter()
//             .map(|(idx, e)| format!("#{}: {}", idx, e))
//             .collect();
//         format!(
//             "Found {} errors in {}:\n{}",
//             errors.len(),
//             item_name,
//             summary.join("\n")
//         )
//     }
// }

// Helper function to show validation dialog
// pub fn show_validation_dialog(message: &str) {
//     rfd::MessageDialog::new()
//         .set_title("Validation Errors")
//         .set_description(message)
//         .show();
// }

/// Macro to handle the complete editor message routing
#[macro_export]
macro_rules! handle_editor_messages {
    ($message:ident, $app:ident, $($pattern:pat => $handler:expr),*) => {
        match $message {
            $($pattern => $handler),*,
            _ => {
                eprintln!("Unhandled message: {:?}", $message);
                Task::none()
            }
        }
    };
}

// use crate::app::App;
// use crate::loading_state::LoadingState;
// use crate::message::Message;
// use crate::message::editor::EditorMessage;
// use crate::view::editor::{SpreadsheetMessage, PaneResizeEvent, Pane};
// use anyhow::{Context, Result};
// use dispel_core::{editable::EditableRecord, Extractor};
// use iced::Task;
// use std::path::PathBuf;

// pub fn load_catalog<R: Extractor + Clone + 'static>(
//     app: &mut App,
//     state: &mut crate::generic_editor::GenericEditorState<R>,
//     item_name: &str,
//     db_path: &str,
// ) -> Task<Message> {
//     if app.state.shared_game_path.is_empty() {
//         state.status_msg = "Please select game path first".to_string();
//         return Task::none();
//     }

//     state.loading_state = LoadingState::Loading;
//     state.status_msg = format!("Loading {}...", item_name);

//     let game_path = app.state.shared_game_path.clone();
//     let path = PathBuf::from(&game_path).join(db_path);

//     Task::perform(
//         async move {
//             R::read_file(&path).context(format!("Failed to load {}", item_name))
//         },
//         move |result: Result<Vec<R>>| {
//             Message::Editor(EditorMessage::CatalogLoaded(result.context(item_name)))
//         },
//     )
// }

// pub fn handle_catalog_loaded<R: EditableRecord + Extractor>(
//     state: &mut crate::generic_editor::GenericEditorState<R>,
//     item_name: &str,
//     result: Result<Vec<R>>,
// ) -> Option<Task<Message>> {
//     state.loading_state = LoadingState::Loaded(());

//     match result {
//         Ok(catalog) => {
//             state.catalog = Some(catalog.clone());
//             state.status_msg = format!("Loaded {} {}", catalog.len(), item_name);
//             state.refresh();
//             state.init_pane_state();
//             None
//         }
//         Err(e) => {
//             state.status_msg = format!("Failed to load {}: {}", item_name, e);
//             None
//         }
//     }
// }

// pub fn save_catalog<R: EditableRecord + Extractor + Clone + Send + 'static>(
//     app: &mut App,
//     state: &mut crate::generic_editor::GenericEditorState<R>,
//     item_name: &str,
//     db_path: &str,
// ) -> Task<Message> {
//     if app.state.shared_game_path.is_empty() {
//         state.status_msg = "Please select game path first".to_string();
//         return Task::none();
//     }

//     if let Some(catalog) = &state.catalog {
//         if let Some(errors) = state.validate() {
//             let message = format_errors(errors, item_name);
//             state.status_msg = message.clone();
//             show_validation_dialog(&message);
//             return Task::none();
//         }

//         state.loading_state = LoadingState::Loading;
//         state.status_msg = format!("Saving {}...", item_name);

//         let game_path = app.state.shared_game_path.clone();
//         let catalog = catalog.clone();
//         let path = PathBuf::from(&game_path).join(db_path);

//         return Task::perform(
//             async move {
//                 R::write_file(&path, &catalog).context(format!("Failed to save {}", item_name))
//             },
//             move |result: Result<()>| {
//                 Message::Editor(EditorMessage::Saved(result.context(item_name)))
//             },
//         );
//     }

//     state.status_msg = format!("No {} to save", item_name);
//     Task::none()
// }

// pub fn handle_saved<R: EditableRecord>(
//     state: &mut crate::generic_editor::GenericEditorState<R>,
//     item_name: &str,
//     result: Result<()>,
// ) {
//     state.loading_state = LoadingState::Loaded(());

//     match result {
//         Ok(_) => {
//             state.status_msg = format!("Saved {} successfully", item_name);
//         }
//         Err(e) => {
//             state.status_msg = format!("Failed to save {}: {}", item_name, e);
//         }
//     }
// }

// pub fn select_item<R: EditableRecord>(
//     state: &mut crate::generic_editor::GenericEditorState<R>,
//     item_name: &str,
//     index: usize,
// ) {
//     if let Some(catalog) = &state.catalog {
//         if index < catalog.len() {
//             state.selected_idx = Some(index);
//             state.status_msg = format!("Selected {} #{}", item_name, index);
//             return;
//         }
//     }
//     state.status_msg = format!("Invalid {} index: {}", item_name, index);
// }

// pub fn update_field<R: EditableRecord>(
//     state: &mut crate::generic_editor::GenericEditorState<R>,
//     item_name: &str,
//     index: usize,
//     field: String,
//     value: String,
// ) {
//     if state.update_field(index, &field, value) {
//         state.status_msg = format!("Updated {} field '{}'", item_name, field);
//     } else {
//         state.status_msg = format!("Invalid value for field '{}'", field);
//     }
// }

// fn state_status(item: &str, action: &str) {
//     eprintln!("[{}] {}", item, action);
// }

// pub fn handle_pane_resize<R: EditableRecord>(
//     state: &mut crate::generic_editor::GenericEditorState<R>,
//     event: PaneResizeEvent,
// ) {
//     if let Some(pane_state) = &mut state.pane_state {
//         pane_state.resize(event.split, event.ratio);
//     }
// }

// pub fn handle_pane_click<R: EditableRecord>(
//     state: &mut crate::generic_editor::GenericEditorState<R>,
//     pane: Pane,
// ) {
//     state.pane_focus = Some(pane);
// }

// fn format_errors(errors: Vec<(usize, String)>, item_name: &str) -> String {
//     if errors.len() > 5 {
//         let summary: Vec<_> = errors.iter().take(5).map(|(idx, e)| format!("#{}: {}", idx, e)).collect();
//         format!(
//             "Found {} errors in {}:\n{}\n... and {} more",
//             errors.len(),
//             item_name,
//             summary.join("\n"),
//             errors.len() - 5
//         )
//     } else {
//         let summary: Vec<_> = errors.iter().map(|(idx, e)| format!("#{}: {}", idx, e)).collect();
//         format!(
//             "Found {} errors in {}:\n{}",
//             errors.len(),
//             item_name,
//             summary.join("\n")
//         )
//     }
// }

// fn show_validation_dialog(message: &str) {
//         .set_description(message)
//         .show();
// }
