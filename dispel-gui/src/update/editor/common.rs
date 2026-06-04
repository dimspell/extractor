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
            ss: $app.state.editors.$field.spreadsheet,
            catalog: || &$app.state.editors.$field.state.catalog,
            make_inspector: |idx| $app.state.editors.$field.make_inspector_textarea_contents(idx),
            unique_values: |col| $app.state.editors.$field.unique_values_for_column(col),
            status_msg: &mut $app.state.editors.$field.status_msg,
            compute_caches: {
                if let Some(c) = &$app.state.editors.$field.state.catalog {
                    let c = c.clone();
                    let lookups = &$app.state.lookups;
                    $app.state.editors.$field.spreadsheet.compute_all_caches(&c, lookups);
                }
            },
            toggle_inspector_extra: {
                if $app.state.editors.$field.spreadsheet.show_inspector {
                    if let Some(orig_idx) = $app.state.editors.$field.spreadsheet.selected_orig {
                        $app.state.editors.$field.spreadsheet.inspector_textarea_contents =
                            $app.state.editors.$field.make_inspector_textarea_contents(orig_idx);
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
                    $app.state.editors.$tabbed_editor.editors.get_mut(&$tab_id),
                    $app.state.editors.$tabbed_editor.spreadsheets.get_mut(&$tab_id),
                ) {
                    $crate::handle_spreadsheet_messages_inner! {
                        app: $app,
                        ss: ss,
                        catalog: || &ed.editor.catalog,
                        make_inspector: |idx| ed.editor.make_inspector_textarea_contents(idx),
                        unique_values: |col| ed.editor.unique_values_for_column(col),
                        status_msg: &mut ed.editor.status_msg,
                        compute_caches: {
                            let ss2 = $app.state.editors.$tabbed_editor.spreadsheets.get_mut(&$tab_id);
                            let ed2 = $app.state.editors.$tabbed_editor.editors.get_mut(&$tab_id);
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



