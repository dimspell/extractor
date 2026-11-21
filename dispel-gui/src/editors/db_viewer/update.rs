// Viewer message handlers

use crate::app::App;
use crate::components::loading_state;
use crate::components::utils::browse_file;
use crate::editors::db_viewer::db;
use crate::editors::db_viewer::state::PAGE_SIZE;
use crate::message::viewer::ViewerMessage;
use iced::Task;

pub fn handle(message: ViewerMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        ViewerMessage::DbPathChanged(path) => {
            // Handle DB path change
            let path_str = path.to_string();
            app.state.editors.viewer.db_path = path_str.clone();
            app.state.editors.viewer.status_msg = format!("Database path set to: {}", path_str);
            app.state.editors.viewer.status_icon = None;
            Task::none()
        }
        ViewerMessage::BrowseDb => {
            // Browse for database file
            browse_file("viewer_db")
        }
        ViewerMessage::Connect => {
            // Connect to database and load tables
            if app.state.editors.viewer.db_path.is_empty() {
                app.state.editors.viewer.status_msg = "Please select a database file first".into();
                app.state.editors.viewer.status_icon = None;
                return Task::none();
            }

            app.state.editors.viewer.loading_state = loading_state::LoadingState::Loading;
            app.state.editors.viewer.status_msg = "Connecting to database...".into();
            app.state.editors.viewer.status_icon = None;

            let db_path = app.state.editors.viewer.db_path.clone();
            Task::perform(async move { db::list_tables(&db_path) }, |result| {
                crate::message::Message::Viewer(ViewerMessage::TablesLoaded(result))
            })
        }
        ViewerMessage::TablesLoaded(result) => {
            // This is handled in the Connect message above
            // The result is processed there, so this is just a fallback
            match result {
                Ok(tables) => {
                    let tables_clone = tables.clone();
                    app.state.editors.viewer.tables = tables_clone;
                    app.state.editors.viewer.active_table = None;
                    app.state.editors.viewer.rows.clear();
                    app.state.editors.viewer.columns.clear();
                    app.state.editors.viewer.status_msg =
                        format!("Connected successfully. Found {} tables.", tables.len());
                    app.state.editors.viewer.status_icon = None;
                }
                Err(e) => {
                    app.state.editors.viewer.status_msg = format!("Error loading tables: {}", e);
                    app.state.editors.viewer.status_icon = Some(lucide_icons::Icon::X);
                }
            }
            app.state.editors.viewer.loading_state = loading_state::LoadingState::Loaded(());
            Task::none()
        }
        ViewerMessage::SelectTable(table) => {
            app.state.editors.viewer.active_table = Some(table.clone());
            app.state.editors.viewer.page = 0;
            app.state.editors.viewer.search.clear();
            app.state.editors.viewer.sort_col = None;
            app.state.editors.viewer.pending_edits.clear();
            app.state.editors.viewer.editing_cell = None;
            app.state.editors.viewer.sql_mode = false;
            app.state.editors.viewer.sql_query = format!("SELECT * FROM \"{}\"", table);
            crate::editors::db_viewer::fetch::fetch_table_data(&mut app.state.editors.viewer)
        }
        ViewerMessage::DataLoaded(result) => {
            app.state.editors.viewer.loading_state =
                crate::components::loading_state::LoadingState::Loaded(());
            match result {
                Ok(qr) => {
                    app.state.editors.viewer.columns = qr.columns;
                    app.state.editors.viewer.rows = qr.rows;
                    app.state.editors.viewer.total_rows = qr.total_rows;
                    let page_start = app.state.editors.viewer.page * PAGE_SIZE + 1;
                    let page_end =
                        (page_start - 1 + app.state.editors.viewer.rows.len()).max(page_start - 1);
                    app.state.editors.viewer.status_msg = format!(
                        "Showing {}-{} of {} rows",
                        page_start, page_end, app.state.editors.viewer.total_rows
                    );
                    app.state.editors.viewer.status_icon = None;
                }
                Err(e) => {
                    app.state.editors.viewer.status_msg = format!("Query error: {}", e);
                    app.state.editors.viewer.status_icon = Some(lucide_icons::Icon::X);
                }
            }
            Task::none()
        }
        ViewerMessage::Search(query) => {
            // Search in current table
            if app.state.editors.viewer.active_table.is_none() {
                app.state.editors.viewer.status_msg = "No active table selected".into();
                app.state.editors.viewer.status_icon = None;
                return Task::none();
            }

            app.state.editors.viewer.search = query;
            app.state.editors.viewer.page = 0;
            crate::editors::db_viewer::fetch::fetch_table_data(&mut app.state.editors.viewer)
        }
        ViewerMessage::SortColumn(column_index) => {
            // Sort by specified column
            if app.state.editors.viewer.sort_col == Some(column_index) {
                app.state.editors.viewer.sort_dir = app.state.editors.viewer.sort_dir.toggle();
            } else {
                app.state.editors.viewer.sort_col = Some(column_index);
                app.state.editors.viewer.sort_dir = db::SortDir::Asc;
            }
            app.state.editors.viewer.page = 0;
            crate::editors::db_viewer::fetch::fetch_table_data(&mut app.state.editors.viewer)
        }
        ViewerMessage::NextPage => {
            // Navigate to next page
            let max_page = app.state.editors.viewer.total_rows.saturating_sub(1) / PAGE_SIZE;
            if app.state.editors.viewer.page < max_page {
                app.state.editors.viewer.page += 1;
                return crate::editors::db_viewer::fetch::fetch_table_data(
                    &mut app.state.editors.viewer,
                );
            }
            Task::none()
        }
        ViewerMessage::PrevPage => {
            // Navigate to previous page
            if app.state.editors.viewer.page > 0 {
                app.state.editors.viewer.page -= 1;
                return crate::editors::db_viewer::fetch::fetch_table_data(
                    &mut app.state.editors.viewer,
                );
            }
            Task::none()
        }
        ViewerMessage::CellClick(row, col) => {
            // Handle cell click for editing
            app.state.editors.viewer.status_msg = format!("Cell clicked: row {}, col {}", row, col);
            app.state.editors.viewer.status_icon = None;
            // Confirm previous edit if any
            if let Some((pr, pc)) = app.state.editors.viewer.editing_cell
                && (!app.state.editors.viewer.edit_buffer.is_empty()
                    || app
                        .state
                        .editors
                        .viewer
                        .rows
                        .get(pr)
                        .and_then(|row| row.get(pc).map(|v| v.as_str()))
                        != Some(&app.state.editors.viewer.edit_buffer))
            {
                let original = app
                    .state
                    .editors
                    .viewer
                    .rows
                    .get(pr)
                    .and_then(|row| row.get(pc))
                    .cloned()
                    .unwrap_or_default();
                if app.state.editors.viewer.edit_buffer != original {
                    app.state
                        .editors
                        .viewer
                        .pending_edits
                        .insert((pr, pc), app.state.editors.viewer.edit_buffer.clone());
                }
            }
            let val = app
                .state
                .editors
                .viewer
                .rows
                .get(row)
                .and_then(|row| row.get(col))
                .cloned()
                .unwrap_or_default();
            app.state.editors.viewer.editing_cell = Some((row, col));
            app.state.editors.viewer.edit_buffer = val;
            Task::none()
        }
        ViewerMessage::CellEdit(value) => {
            // Handle cell edit
            app.state.editors.viewer.status_msg = format!("Editing cell: {}", value);
            app.state.editors.viewer.status_icon = None;
            app.state.editors.viewer.edit_buffer = value;
            Task::none()
        }
        ViewerMessage::CellConfirm => {
            // Confirm cell edit
            app.state.editors.viewer.status_msg = "Cell edit staged for commit".into();
            app.state.editors.viewer.status_icon = None;
            if let Some((r, c)) = app.state.editors.viewer.editing_cell {
                let original = app
                    .state
                    .editors
                    .viewer
                    .rows
                    .get(r)
                    .and_then(|row| row.get(c))
                    .cloned()
                    .unwrap_or_default();
                if app.state.editors.viewer.edit_buffer != original {
                    app.state
                        .editors
                        .viewer
                        .pending_edits
                        .insert((r, c), app.state.editors.viewer.edit_buffer.clone());
                }
            }
            app.state.editors.viewer.editing_cell = None;
            Task::none()
        }
        ViewerMessage::CellCancel => {
            // Cancel cell edit
            app.state.editors.viewer.status_msg = "Cell edit cancelled".into();
            app.state.editors.viewer.status_icon = None;
            app.state.editors.viewer.editing_cell = None;
            Task::none()
        }
        ViewerMessage::Commit => {
            // Commit all edits to database
            if app.state.editors.viewer.pending_edits.is_empty() {
                app.state.editors.viewer.status_msg = "No edits to commit".into();
                app.state.editors.viewer.status_icon = None;
                return Task::none();
            }

            app.state.editors.viewer.status_msg = "Committing edits...".into();
            app.state.editors.viewer.status_icon = None;
            app.state.editors.viewer.loading_state = loading_state::LoadingState::Loading;

            let db_path = app.state.editors.viewer.db_path.clone();
            let table = app.state.editors.viewer.active_table.clone();
            let edits = app.state.editors.viewer.pending_edits.clone();
            let columns = app.state.editors.viewer.columns.clone();
            let rows = app.state.editors.viewer.rows.clone();

            app.state.editors.viewer.loading_state = loading_state::LoadingState::Loading;
            Task::perform(
                async move {
                    if let Some(table_name) = table {
                        db::commit_edits(&db_path, &table_name, &columns, &rows, &edits)
                    } else {
                        Err("No active table".into())
                    }
                },
                |result| crate::message::Message::Viewer(ViewerMessage::CommitDone(result)),
            )
        }
        ViewerMessage::CommitDone(result) => {
            // This is handled in the Commit message above
            app.state.editors.viewer.loading_state = loading_state::LoadingState::Loaded(());
            match result {
                Ok(n) => {
                    // Apply edits to local rows
                    for ((r, c), val) in &app.state.editors.viewer.pending_edits {
                        if let Some(row) = app.state.editors.viewer.rows.get_mut(*r)
                            && let Some(cell) = row.get_mut(*c)
                        {
                            *cell = val.clone();
                        }
                    }
                    app.state.editors.viewer.pending_edits.clear();
                    app.state.editors.viewer.status_msg = format!("Committed {} row(s)", n);
                    app.state.editors.viewer.status_icon = Some(lucide_icons::Icon::Check);
                }
                Err(e) => {
                    app.state.editors.viewer.status_msg = format!("Commit failed: {}", e);
                    app.state.editors.viewer.status_icon = Some(lucide_icons::Icon::X);
                }
            }
            Task::none()
        }
        ViewerMessage::ToggleSql => {
            // Toggle SQL mode
            app.state.editors.viewer.sql_mode = !app.state.editors.viewer.sql_mode;
            Task::none()
        }
        ViewerMessage::SqlChanged(sql) => {
            // Handle SQL query change
            app.state.editors.viewer.sql_query = sql;
            app.state.editors.viewer.status_msg = "SQL query updated".into();
            app.state.editors.viewer.status_icon = None;
            Task::none()
        }
        ViewerMessage::RunSql => {
            app.state.editors.viewer.page = 0;
            app.state.editors.viewer.pending_edits.clear();
            app.state.editors.viewer.editing_cell = None;
            crate::editors::db_viewer::fetch::fetch_sql_data(&mut app.state.editors.viewer)
        }
        ViewerMessage::ExportCsv => {
            // Export data to CSV
            if app.state.editors.viewer.rows.is_empty() {
                app.state.editors.viewer.status_msg = "No data to export".into();
                app.state.editors.viewer.status_icon = None;
                return Task::none();
            }

            let cols = app.state.editors.viewer.columns.clone();
            let rows = app.state.editors.viewer.rows.clone();
            app.state.editors.viewer.status_msg = "Exporting to CSV...".into();
            app.state.editors.viewer.status_icon = None;

            Task::perform(
                async move {
                    let handle = rfd::AsyncFileDialog::new()
                        .set_file_name("export.csv")
                        .add_filter("CSV", &["csv"])
                        .save_file()
                        .await;
                    match handle {
                        Some(h) => {
                            let path = h.path().to_string_lossy().to_string();
                            db::export_csv(&path, &cols, &rows).map(|_| path)
                        }
                        None => Err("Cancelled".into()),
                    }
                },
                move |result| crate::message::Message::Viewer(ViewerMessage::CsvSaved(result)),
            )
        }
        ViewerMessage::CsvSaved(result) => {
            match result {
                Ok(p) => {
                    app.state.editors.viewer.status_msg = format!("CSV exported to {}", p);
                    app.state.editors.viewer.status_icon = Some(lucide_icons::Icon::Check);
                }
                Err(e) => {
                    app.state.editors.viewer.status_msg = format!("CSV export failed: {}", e);
                    app.state.editors.viewer.status_icon = Some(lucide_icons::Icon::X);
                }
            }
            Task::none()
        }
        ViewerMessage::RevertEdits => {
            // Revert all unsaved edits
            app.state.editors.viewer.pending_edits.clear();
            app.state.editors.viewer.editing_cell = None;
            app.state.editors.viewer.status_msg = "Reverted all pending edits.".into();
            app.state.editors.viewer.status_icon = None;
            Task::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::workspace::Workspace;

    #[test]
    fn test_cell_confirm_reports_staged_edit() {
        let mut app = App::test_new(Workspace::new());
        let viewer = &mut app.state.editors.viewer;
        viewer.rows = vec![vec!["old value".into()]];
        viewer.editing_cell = Some((0, 0));
        viewer.edit_buffer = "new value".into();

        let _ = handle(ViewerMessage::CellConfirm, &mut app);

        assert_eq!(
            app.state.editors.viewer.status_msg,
            "Cell edit staged for commit"
        );
        assert_eq!(
            app.state.editors.viewer.pending_edits.get(&(0, 0)),
            Some(&"new value".to_string())
        );
        assert_eq!(app.state.editors.viewer.editing_cell, None);
    }
}
