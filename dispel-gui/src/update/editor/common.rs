//! Common Editor Handler Framework
//!
//! Provides macros and helper functions to handle common editor operations
//! with good user feedback and clear error handling.

/// Macro to handle spreadsheet messages generically
/// Reduces ~80-100 lines of repetitive code to ~10 lines per editor
#[macro_export]
macro_rules! handle_spreadsheet_messages {
    ($app:ident, $spreadsheet:ident, $editor:ident, $field_changed_msg:expr, $msg:ident) => {
        use $crate::view::editor::SpreadsheetMessage as SM;
        match $msg {
            SM::ToggleActive => {
                $app.state.$spreadsheet.toggle_active();
                if $app.state.$spreadsheet.active {
                    if let Some(catalog) = &$app.state.$editor.catalog {
                        $app.state.$spreadsheet.init_filter(catalog);
                        $app.state.$spreadsheet.init_pane_state();
                    }
                }
            }
            SM::SortColumn(col) => {
                $app.state.$spreadsheet.toggle_sort(col);
                if let Some(catalog) = &$app.state.$editor.catalog {
                    $app.state.$spreadsheet.apply_sort(catalog);
                }
            }
            SM::FilterChanged(query) => {
                $app.state.$spreadsheet.filter_query = query;
                if let Some(catalog) = &$app.state.$editor.catalog {
                    $app.state.$spreadsheet.apply_filter(catalog);
                }
            }
            SM::ClearFilter => {
                if let Some(catalog) = &$app.state.$editor.catalog {
                    $app.state.$spreadsheet.clear_filter(catalog);
                }
            }
            SM::SetFilterMode(mode) => {
                if let Some(catalog) = &$app.state.$editor.catalog {
                    $app.state.$spreadsheet.set_filter_mode(mode, catalog);
                }
            }
            SM::NavigateNextHighlight => {
                $app.state.$spreadsheet.navigate_next_highlight();
                if let Some(orig_idx) = $app.state.$spreadsheet.current_highlight_orig_idx() {
                    if let Some(fidx) = $app
                        .state
                        .$spreadsheet
                        .filtered_indices
                        .iter()
                        .position(|&i| i == orig_idx)
                    {
                        let y = $app.state.$spreadsheet.scroll_y_for_row(fidx);
                        let x = $app.state.$spreadsheet.horizontal_scroll_offset;
                        return iced::widget::operation::scroll_to(
                            $app.state.$spreadsheet.body_scroll_id.clone(),
                            iced::widget::scrollable::AbsoluteOffset { x, y },
                        );
                    }
                }
            }
            SM::NavigatePrevHighlight => {
                $app.state.$spreadsheet.navigate_prev_highlight();
                if let Some(orig_idx) = $app.state.$spreadsheet.current_highlight_orig_idx() {
                    if let Some(fidx) = $app
                        .state
                        .$spreadsheet
                        .filtered_indices
                        .iter()
                        .position(|&i| i == orig_idx)
                    {
                        let y = $app.state.$spreadsheet.scroll_y_for_row(fidx);
                        let x = $app.state.$spreadsheet.horizontal_scroll_offset;
                        return iced::widget::operation::scroll_to(
                            $app.state.$spreadsheet.body_scroll_id.clone(),
                            iced::widget::scrollable::AbsoluteOffset { x, y },
                        );
                    }
                }
            }
            SM::NavigateUp => {
                if let Some(fidx) = $app.state.$spreadsheet.navigate_up() {
                    if let Some(&orig_idx) = $app.state.$spreadsheet.filtered_indices.get(fidx) {
                        $app.state.$spreadsheet.inspector_textarea_contents = $app
                            .state
                            .$editor
                            .make_inspector_textarea_contents(orig_idx);
                    }
                    let y = $app.state.$spreadsheet.scroll_y_for_row(fidx);
                    let x = $app.state.$spreadsheet.horizontal_scroll_offset;
                    return iced::widget::operation::scroll_to(
                        $app.state.$spreadsheet.body_scroll_id.clone(),
                        iced::widget::scrollable::AbsoluteOffset { x, y },
                    );
                }
            }
            SM::NavigateDown => {
                if let Some(fidx) = $app.state.$spreadsheet.navigate_down() {
                    if let Some(&orig_idx) = $app.state.$spreadsheet.filtered_indices.get(fidx) {
                        $app.state.$spreadsheet.inspector_textarea_contents = $app
                            .state
                            .$editor
                            .make_inspector_textarea_contents(orig_idx);
                    }
                    let y = $app.state.$spreadsheet.scroll_y_for_row(fidx);
                    let x = $app.state.$spreadsheet.horizontal_scroll_offset;
                    return iced::widget::operation::scroll_to(
                        $app.state.$spreadsheet.body_scroll_id.clone(),
                        iced::widget::scrollable::AbsoluteOffset { x, y },
                    );
                }
            }
            SM::NavigateTop => {
                if let Some(fidx) = $app.state.$spreadsheet.navigate_top() {
                    if let Some(&orig_idx) = $app.state.$spreadsheet.filtered_indices.get(fidx) {
                        $app.state.$spreadsheet.inspector_textarea_contents = $app
                            .state
                            .$editor
                            .make_inspector_textarea_contents(orig_idx);
                    }
                    let x = $app.state.$spreadsheet.horizontal_scroll_offset;
                    return iced::widget::operation::scroll_to(
                        $app.state.$spreadsheet.body_scroll_id.clone(),
                        iced::widget::scrollable::AbsoluteOffset { x, y: 0.0 },
                    );
                }
            }
            SM::NavigateBottom => {
                if let Some(fidx) = $app.state.$spreadsheet.navigate_bottom() {
                    if let Some(&orig_idx) = $app.state.$spreadsheet.filtered_indices.get(fidx) {
                        $app.state.$spreadsheet.inspector_textarea_contents = $app
                            .state
                            .$editor
                            .make_inspector_textarea_contents(orig_idx);
                    }
                    let y = $app.state.$spreadsheet.scroll_y_for_row(fidx);
                    let x = $app.state.$spreadsheet.horizontal_scroll_offset;
                    return iced::widget::operation::scroll_to(
                        $app.state.$spreadsheet.body_scroll_id.clone(),
                        iced::widget::scrollable::AbsoluteOffset { x, y },
                    );
                }
            }
            SM::SelectRow(filtered_idx) => {
                $app.state.$spreadsheet.select_row(filtered_idx);
                // Update inspector textarea contents for the selected record if the
                // inspector is already visible; don't auto-open it (user uses the
                // Inspector button) to avoid pane-restructuring lag on first click.
                if let Some(&orig_idx) = $app.state.$spreadsheet.filtered_indices.get(filtered_idx)
                {
                    if $app.state.$spreadsheet.show_inspector {
                        $app.state.$spreadsheet.inspector_textarea_contents = $app
                            .state
                            .$editor
                            .make_inspector_textarea_contents(orig_idx);
                    }
                } else {
                    $app.state.$spreadsheet.inspector_textarea_contents.clear();
                }
            }
            SM::TextAreaChanged(orig_idx, field, action) => {
                if let Some(tc) = $app
                    .state
                    .$spreadsheet
                    .inspector_textarea_contents
                    .get_mut(&field)
                {
                    tc.0.perform(action);
                    let raw = tc.0.text();
                    let new_text = raw.strip_suffix('\n').unwrap_or(&raw).to_string();
                    let msg = $field_changed_msg(orig_idx, field, new_text);
                    return $app.update(msg);
                }
            }
            SM::StartEdit(filtered_idx, col) => {
                if let Some(catalog) = &$app.state.$editor.catalog {
                    $app.state
                        .$spreadsheet
                        .start_editing(filtered_idx, col, catalog);
                }
            }
            SM::EditCellInput(v) => {
                $app.state.$spreadsheet.edit_buffer = v;
            }
            SM::CommitEdit(orig_idx) => {
                if let Some(catalog) = &mut $app.state.$editor.catalog {
                    if let Some(msg) =
                        $app.state
                            .$spreadsheet
                            .commit_edit(catalog, $field_changed_msg, orig_idx)
                    {
                        return $app.update(msg);
                    }
                }
            }
            SM::CancelEdit => {
                // ESC doubles as a resize-cancel when a column drag is in progress.
                if $app.state.$spreadsheet.resizing_column.is_some() {
                    $app.state.$spreadsheet.end_column_resize();
                } else {
                    $app.state.$spreadsheet.cancel_editing();
                }
            }
            SM::ToggleInspector => {
                $app.state.$spreadsheet.toggle_inspector();
                $app.state.$spreadsheet.ensure_inspector_pane();
                // Populate textarea contents now that the inspector is becoming visible.
                if $app.state.$spreadsheet.show_inspector {
                    if let Some(fidx) = $app.state.$spreadsheet.selected_row {
                        if let Some(&orig_idx) = $app.state.$spreadsheet.filtered_indices.get(fidx)
                        {
                            $app.state.$spreadsheet.inspector_textarea_contents = $app
                                .state
                                .$editor
                                .make_inspector_textarea_contents(orig_idx);
                        }
                    }
                }
            }
            SM::CloseInspector => {
                $app.state.$spreadsheet.show_inspector = false;
                $app.state.$spreadsheet.ensure_inspector_pane();
            }
            SM::ExportCsv => {
                if let Some(catalog) = &$app.state.$editor.catalog {
                    match $app.state.$spreadsheet.to_csv_bytes(catalog) {
                        Ok(bytes) => {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_file_name("export.csv")
                                .add_filter("CSV", &["csv"])
                                .save_file()
                            {
                                match std::fs::write(&path, &bytes) {
                                    Ok(_) => {
                                        $app.state.$editor.status_msg =
                                            format!("Exported CSV to {}", path.display());
                                    }
                                    Err(e) => {
                                        $app.state.$editor.status_msg =
                                            format!("CSV export failed: {}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            $app.state.$editor.status_msg = format!("CSV export failed: {}", e);
                        }
                    }
                }
            }
            SM::CsvExported(result) => match result {
                Ok(path) => {
                    $app.state.$editor.status_msg = format!("Exported CSV to {}", path.display());
                }
                Err(e) if e == "cancelled" => {}
                Err(e) => {
                    $app.state.$editor.status_msg = format!("CSV export failed: {}", e);
                }
            },
            SM::BodyScrolled(offset, viewport_height) => {
                $app.state.$spreadsheet.horizontal_scroll_offset = offset.x;
                $app.state.$spreadsheet.vertical_scroll_offset = offset.y;
                $app.state.$spreadsheet.viewport_height = viewport_height;
                return iced::widget::operation::scroll_to(
                    $app.state.$spreadsheet.header_scroll_id.clone(),
                    iced::widget::scrollable::AbsoluteOffset {
                        x: offset.x,
                        y: 0.0,
                    },
                );
            }
            SM::StartResizeColumn(col) => {
                $app.state.$spreadsheet.begin_column_resize(col);
            }
            SM::ResizeColumnCursor(x) => {
                $app.state.$spreadsheet.update_column_resize(x);
            }
            SM::EndResizeColumn => {
                $app.state.$spreadsheet.end_column_resize();
            }
            SM::ResetColumnWidth(col) => {
                $app.state.$spreadsheet.reset_column_width(col);
            }
            SM::OpenColumnFilter(col) => {
                // Toggle: second click on the same column closes the dropdown.
                if $app.state.$spreadsheet.active_column_filter == Some(col) {
                    $app.state.$spreadsheet.active_column_filter = None;
                } else {
                    $app.state.$spreadsheet.column_filter_options =
                        $app.state.$editor.unique_values_for_column(col);
                    $app.state.$spreadsheet.active_column_filter = Some(col);
                }
            }
            SM::ApplyColumnFilter(col, value) => {
                $app.state.$spreadsheet.column_filters.insert(col, value);
                $app.state.$spreadsheet.active_column_filter = None;
                if let Some(catalog) = &$app.state.$editor.catalog {
                    $app.state.$spreadsheet.apply_filter(catalog);
                    $app.state.$spreadsheet.apply_sort(catalog);
                }
            }
            SM::ClearColumnFilter(col) => {
                if let Some(catalog) = &$app.state.$editor.catalog {
                    $app.state.$spreadsheet.clear_column_filter(col, catalog);
                    $app.state.$spreadsheet.apply_sort(catalog);
                }
            }
        }
    };
}

/// Macro to handle spreadsheet messages for tab-based editors (NpcRef, MonsterRef, etc.)
#[macro_export]
macro_rules! handle_spreadsheet_messages_tab {
    ($app:ident, $spreadsheets:ident, $editors:ident, $tab_id:expr, $field_changed_msg:expr, $msg:ident) => {
        use $crate::view::editor::SpreadsheetMessage as SM;
        match $msg {
            SM::CommitEdit(orig_idx) => {
                let result_msg = {
                    let ss = $app.state.$spreadsheets.get_mut($tab_id);
                    let ed = $app.state.$editors.get_mut($tab_id);
                    if let (Some(ss), Some(ed)) = (ss, ed) {
                        if let Some(catalog) = &mut ed.editor.catalog {
                            ss.commit_edit(catalog, $field_changed_msg, orig_idx)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };
                if let Some(msg) = result_msg {
                    return $app.update(msg);
                }
            }
            other => {
                let ss = $app.state.$spreadsheets.get_mut($tab_id);
                let ed = $app.state.$editors.get_mut($tab_id);
                if let (Some(ss), Some(ed)) = (ss, ed) {
                    match other {
                        SM::ToggleActive => {
                            ss.toggle_active();
                            if ss.active {
                                if let Some(c) = &ed.editor.catalog {
                                    ss.init_filter(c);
                                    ss.init_pane_state();
                                }
                            }
                        }
                        SM::SortColumn(col) => {
                            ss.toggle_sort(col);
                            if let Some(c) = &ed.editor.catalog {
                                ss.apply_sort(c);
                            }
                        }
                        SM::FilterChanged(query) => {
                            ss.filter_query = query;
                            if let Some(c) = &ed.editor.catalog {
                                ss.apply_filter(c);
                            }
                        }
                        SM::ClearFilter => {
                            if let Some(c) = &ed.editor.catalog {
                                ss.clear_filter(c);
                            }
                        }
                        SM::SetFilterMode(mode) => {
                            if let Some(c) = &ed.editor.catalog {
                                ss.set_filter_mode(mode, c);
                            }
                        }
                        SM::NavigateNextHighlight => {
                            ss.navigate_next_highlight();
                            if let Some(orig_idx) = ss.current_highlight_orig_idx() {
                                if let Some(fidx) =
                                    ss.filtered_indices.iter().position(|&i| i == orig_idx)
                                {
                                    let y = ss.scroll_y_for_row(fidx);
                                    let x = ss.horizontal_scroll_offset;
                                    return iced::widget::operation::scroll_to(
                                        ss.body_scroll_id.clone(),
                                        iced::widget::scrollable::AbsoluteOffset { x, y },
                                    );
                                }
                            }
                        }
                        SM::NavigatePrevHighlight => {
                            ss.navigate_prev_highlight();
                            if let Some(orig_idx) = ss.current_highlight_orig_idx() {
                                if let Some(fidx) =
                                    ss.filtered_indices.iter().position(|&i| i == orig_idx)
                                {
                                    let y = ss.scroll_y_for_row(fidx);
                                    let x = ss.horizontal_scroll_offset;
                                    return iced::widget::operation::scroll_to(
                                        ss.body_scroll_id.clone(),
                                        iced::widget::scrollable::AbsoluteOffset { x, y },
                                    );
                                }
                            }
                        }
                        SM::NavigateUp => {
                            if let Some(fidx) = ss.navigate_up() {
                                if let Some(&orig_idx) = ss.filtered_indices.get(fidx) {
                                    ss.inspector_textarea_contents =
                                        ed.editor.make_inspector_textarea_contents(orig_idx);
                                }
                                let y = ss.scroll_y_for_row(fidx);
                                let x = ss.horizontal_scroll_offset;
                                return iced::widget::operation::scroll_to(
                                    ss.body_scroll_id.clone(),
                                    iced::widget::scrollable::AbsoluteOffset { x, y },
                                );
                            }
                        }
                        SM::NavigateDown => {
                            if let Some(fidx) = ss.navigate_down() {
                                if let Some(&orig_idx) = ss.filtered_indices.get(fidx) {
                                    ss.inspector_textarea_contents =
                                        ed.editor.make_inspector_textarea_contents(orig_idx);
                                }
                                let y = ss.scroll_y_for_row(fidx);
                                let x = ss.horizontal_scroll_offset;
                                return iced::widget::operation::scroll_to(
                                    ss.body_scroll_id.clone(),
                                    iced::widget::scrollable::AbsoluteOffset { x, y },
                                );
                            }
                        }
                        SM::NavigateTop => {
                            if let Some(_fidx) = ss.navigate_top() {
                                let x = ss.horizontal_scroll_offset;
                                return iced::widget::operation::scroll_to(
                                    ss.body_scroll_id.clone(),
                                    iced::widget::scrollable::AbsoluteOffset { x, y: 0.0 },
                                );
                            }
                        }
                        SM::NavigateBottom => {
                            if let Some(fidx) = ss.navigate_bottom() {
                                let y = ss.scroll_y_for_row(fidx);
                                let x = ss.horizontal_scroll_offset;
                                return iced::widget::operation::scroll_to(
                                    ss.body_scroll_id.clone(),
                                    iced::widget::scrollable::AbsoluteOffset { x, y },
                                );
                            }
                        }
                        SM::SelectRow(filtered_idx) => {
                            ss.select_row(filtered_idx);
                            if let Some(&orig_idx) = ss.filtered_indices.get(filtered_idx) {
                                if ss.show_inspector {
                                    ss.inspector_textarea_contents =
                                        ed.editor.make_inspector_textarea_contents(orig_idx);
                                }
                            } else {
                                ss.inspector_textarea_contents.clear();
                            }
                        }
                        SM::TextAreaChanged(orig_idx, field, action) => {
                            if let Some(tc) = ss.inspector_textarea_contents.get_mut(&field) {
                                tc.0.perform(action);
                                let raw = tc.0.text();
                                let new_text = raw.strip_suffix('\n').unwrap_or(&raw).to_string();
                                let msg = $field_changed_msg(orig_idx, field, new_text);
                                return $app.update(msg);
                            }
                        }
                        SM::StartEdit(filtered_idx, col) => {
                            if let Some(c) = &ed.editor.catalog {
                                ss.start_editing(filtered_idx, col, c);
                            }
                        }
                        SM::EditCellInput(v) => ss.edit_buffer = v,
                        SM::CancelEdit => {
                            if ss.resizing_column.is_some() {
                                ss.end_column_resize();
                            } else {
                                ss.cancel_editing();
                            }
                        }
                        SM::ToggleInspector => {
                            ss.toggle_inspector();
                            ss.ensure_inspector_pane();
                        }
                        SM::CloseInspector => {
                            ss.show_inspector = false;
                            ss.ensure_inspector_pane();
                        }
                        SM::ExportCsv => {
                            if let Some(c) = &ed.editor.catalog {
                                if let Ok(bytes) = ss.to_csv_bytes(c) {
                                    // Best-effort: spawn a blocking write in a detached task.
                                    // Tab editors don't yet route back CsvExported; use a
                                    // synchronous save dialog instead.
                                    if let Some(path) = rfd::FileDialog::new()
                                        .set_file_name("export.csv")
                                        .add_filter("CSV", &["csv"])
                                        .save_file()
                                    {
                                        if let Err(e) = std::fs::write(&path, &bytes) {
                                            ed.editor.status_msg =
                                                format!("CSV export failed: {}", e);
                                        } else {
                                            ed.editor.status_msg =
                                                format!("Exported CSV to {}", path.display());
                                        }
                                    }
                                }
                            }
                        }
                        SM::CsvExported(result) => match result {
                            Ok(path) => {
                                ed.editor.status_msg =
                                    format!("Exported CSV to {}", path.display());
                            }
                            Err(e) if e == "cancelled" => {}
                            Err(e) => {
                                ed.editor.status_msg = format!("CSV export failed: {}", e);
                            }
                        },
                        SM::BodyScrolled(offset, viewport_height) => {
                            ss.horizontal_scroll_offset = offset.x;
                            ss.viewport_height = viewport_height;
                            return iced::widget::operation::scroll_to(
                                ss.header_scroll_id.clone(),
                                iced::widget::scrollable::AbsoluteOffset {
                                    x: offset.x,
                                    y: 0.0,
                                },
                            );
                        }
                        SM::StartResizeColumn(col) => ss.begin_column_resize(col),
                        SM::ResizeColumnCursor(x) => ss.update_column_resize(x),
                        SM::EndResizeColumn => ss.end_column_resize(),
                        SM::ResetColumnWidth(col) => ss.reset_column_width(col),
                        SM::OpenColumnFilter(col) => {
                            if ss.active_column_filter == Some(col) {
                                ss.active_column_filter = None;
                            } else {
                                ss.column_filter_options = ed.editor.unique_values_for_column(col);
                                ss.active_column_filter = Some(col);
                            }
                        }
                        SM::ApplyColumnFilter(col, value) => {
                            ss.column_filters.insert(col, value);
                            ss.active_column_filter = None;
                            if let Some(catalog) = &ed.editor.catalog {
                                ss.apply_filter(catalog);
                                ss.apply_sort(catalog);
                            }
                        }
                        SM::ClearColumnFilter(col) => {
                            if let Some(catalog) = &ed.editor.catalog {
                                ss.clear_column_filter(col, catalog);
                                ss.apply_sort(catalog);
                            }
                        }
                        SM::CommitEdit(_) => unreachable!(),
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
                $app.state.$editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }

            $app.state.$editor.loading_state = $crate::loading_state::LoadingState::Loading;
            $app.state.$editor.status_msg = concat!($item_name, " catalog...").into();

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
        $app.state.$editor.loading_state = $crate::loading_state::LoadingState::Loaded(());
        match $result {
            Ok(catalog) => {
                $app.state.$editor.catalog = Some(catalog.clone());
                $app.state.$editor.status_msg = format!(
                    concat!($item_name, " catalog loaded: {} {}"),
                    catalog.len(),
                    $item_name
                );
                $app.state.$editor.refresh();
                $app.state.$editor.init_pane_state();
                Task::none()
            }
            Err(e) => {
                let msg = format!(concat!("Failed to load ", $item_name, ": {}"), e);
                $app.state.$editor.status_msg = msg.clone();
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
        $app.state.$editor.update_field($index, &$field, $value);
        Task::none()
    }};
}

/// Macro to handle save messages
#[macro_export]
macro_rules! handle_save {
    ($app:ident, $editor:ident, $item_name:expr, $save_method:expr, $saved_variant:expr) => {{
        if $app.state.shared_game_path.is_empty() {
            $app.state.$editor.status_msg = "Please select game path first.".into();
            return Task::none();
        }

        $app.state.$editor.loading_state = $crate::loading_state::LoadingState::Loading;
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
        if let Some(ref mut ps) = $app.state.$editor.pane_state {
            ps.resize($event.split, $event.ratio);
        }
        Task::none()
    }};
}

/// Macro to handle pane clicked messages
#[macro_export]
macro_rules! handle_pane_clicked {
    ($app:ident, $editor:ident, $pane:ident) => {{
        $app.state.$editor.pane_focus = Some($pane);
        Task::none()
    }};
}

/// Macro to handle saved messages
#[macro_export]
macro_rules! handle_saved {
    ($app:ident, $editor:ident, $item_name:expr, $result:expr) => {{
        $app.state.$editor.loading_state = $crate::loading_state::LoadingState::Loaded(());
        match $result {
            Ok(_) => {
                $app.state.$editor.status_msg = format!(concat!($item_name, " saved successfully"));
            }
            Err(e) => {
                $app.state.$editor.status_msg =
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
        $app.state.$editor.status_msg = $error_msg.into();
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
