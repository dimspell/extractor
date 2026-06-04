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
                return Task::none();
            }

            app.state.editors.viewer.loading_state = loading_state::LoadingState::Loading;
            app.state.editors.viewer.status_msg = "Connecting to database...".into();

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
                }
                Err(e) => {
                    app.state.editors.viewer.status_msg = format!("Error loading tables: {}", e);
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
            app.fetch_viewer_data()
            // ----

            // // Select table and load its data
            // if app.state.editors.viewer.tables.is_empty() {
            //     app.state.editors.viewer.status_msg = "No tables available".into();
            //     return Task::none();
            // }
            //
            // app.state.editors.viewer.loading_state = loading_state::LoadingState::Loading;
            // app.state.editors.viewer.status_msg = format!("Loading table: {}", table);
            // app.state.editors.viewer.active_table = Some(table.clone());
            //
            // let db_path = app.state.editors.viewer.db_path.clone();
            // let table_clone = table.clone();
            // return Task::perform(
            //     async move {
            //         let columns = db::table_columns(&db_path, &table_clone)?;
            //         let query =
            //             db::build_table_query(&table_clone, &columns, "", None, SortDir::Asc);
            //         let data = db::execute_query(&db_path, &query, 100, 0)?;
            //         Ok(data.rows)
            //     },
            //     |result: Result<Vec<Vec<String>>, String>| {
            //         app.state.editors.viewer.loading_state = loading_state::LoadingState::Loaded(());
            //         match result {
            //             Ok(rows) => {
            //                 let columns =
            //                     db::table_columns(&db_path, &table_clone).unwrap_or_default();
            //                 app.state.editors.viewer.columns = columns;
            //                 app.state.editors.viewer.rows = rows.clone();
            //                 app.state.editors.viewer.total_rows = app.state.editors.viewer.rows.len();
            //                 app.state.editors.viewer.status_msg =
            //                     format!("Loaded {} rows from table: {}", rows.len(), table_clone);
            //             }
            //             Err(e) => {
            //                 app.state.editors.viewer.status_msg = format!("Failed to load table: {}", e);
            //                 app.state.editors.viewer.active_table = None;
            //             }
            //         }
            //         crate::message::Message::Viewer(ViewerMessage::DataLoaded(Ok(app
            //             .state
            //             .editors.viewer
            //             .rows
            //             .clone())))
            //     },
            // );
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
                }
                Err(e) => {
                    app.state.editors.viewer.status_msg = format!("✖ Query error: {}", e);
                }
            }
            Task::none()
        }
        ViewerMessage::Search(query) => {
            // Search in current table
            if app.state.editors.viewer.active_table.is_none() {
                app.state.editors.viewer.status_msg = "No active table selected".into();
                return Task::none();
            }

            app.state.editors.viewer.search = query;
            app.state.editors.viewer.page = 0;
            app.fetch_viewer_data()
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
            app.fetch_viewer_data()
        }
        ViewerMessage::NextPage => {
            // Navigate to next page
            let max_page = app.state.editors.viewer.total_rows.saturating_sub(1) / PAGE_SIZE;
            if app.state.editors.viewer.page < max_page {
                app.state.editors.viewer.page += 1;
                return app.fetch_viewer_data();
            }
            Task::none()
        }
        ViewerMessage::PrevPage => {
            // Navigate to previous page
            if app.state.editors.viewer.page > 0 {
                app.state.editors.viewer.page -= 1;
                return app.fetch_viewer_data();
            }
            Task::none()
        }
        ViewerMessage::CellClick(row, col) => {
            // Handle cell click for editing
            app.state.editors.viewer.status_msg = format!("Cell clicked: row {}, col {}", row, col);
            // Confirm previous edit if any
            if let Some((pr, pc)) = app.state.editors.viewer.editing_cell {
                if !app.state.editors.viewer.edit_buffer.is_empty()
                    || app
                        .state
                        .editors.viewer
                        .rows
                        .get(pr)
                        .and_then(|row| row.get(pc).map(|v| v.as_str()))
                        != Some(&app.state.editors.viewer.edit_buffer)
                {
                    let original = app
                        .state
                        .editors.viewer
                        .rows
                        .get(pr)
                        .and_then(|row| row.get(pc))
                        .cloned()
                        .unwrap_or_default();
                    if app.state.editors.viewer.edit_buffer != original {
                        app.state
                            .editors.viewer
                            .pending_edits
                            .insert((pr, pc), app.state.editors.viewer.edit_buffer.clone());
                    }
                }
            }
            let val = app
                .state
                .editors.viewer
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
            app.state.editors.viewer.edit_buffer = value;
            Task::none()
        }
        ViewerMessage::CellConfirm => {
            // Confirm cell edit
            app.state.editors.viewer.status_msg = "Cell edit confirmed (implementation needed)".into();
            if let Some((r, c)) = app.state.editors.viewer.editing_cell {
                let original = app
                    .state
                    .editors.viewer
                    .rows
                    .get(r)
                    .and_then(|row| row.get(c))
                    .cloned()
                    .unwrap_or_default();
                if app.state.editors.viewer.edit_buffer != original {
                    app.state
                        .editors.viewer
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
            app.state.editors.viewer.editing_cell = None;
            Task::none()
        }
        ViewerMessage::Commit => {
            // Commit all edits to database
            if app.state.editors.viewer.pending_edits.is_empty() {
                app.state.editors.viewer.status_msg = "No edits to commit".into();
                return Task::none();
            }

            app.state.editors.viewer.status_msg = "Committing edits...".into();
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
                        if let Some(row) = app.state.editors.viewer.rows.get_mut(*r) {
                            if let Some(cell) = row.get_mut(*c) {
                                *cell = val.clone();
                            }
                        }
                    }
                    app.state.editors.viewer.pending_edits.clear();
                    app.state.editors.viewer.status_msg = format!("✔ Committed {} row(s)", n);
                }
                Err(e) => {
                    app.state.editors.viewer.status_msg = format!("✖ Commit failed: {}", e);
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
            Task::none()
        }
        ViewerMessage::RunSql => {
            app.state.editors.viewer.page = 0;
            app.state.editors.viewer.pending_edits.clear();
            app.state.editors.viewer.editing_cell = None;
            app.fetch_viewer_data_sql()

            // AI:
            // Execute SQL query
            // if app.state.editors.viewer.sql_query.is_empty() {
            //     app.state.editors.viewer.status_msg = "No SQL query to execute".into();
            //     return Task::none();
            // }
            //
            // app.state.editors.viewer.status_msg = "Executing SQL query...".into();
            // app.state.editors.viewer.loading_state = loading_state::LoadingState::Loading;
            //
            // let db_path = app.state.editors.viewer.db_path.clone();
            // let sql = app.state.editors.viewer.sql_query.clone();
            //
            // return Task::perform(
            //     async move { db::execute_query(&db_path, &sql, 100, 0) },
            //     move |result| {
            //         app.state.editors.viewer.loading_state = loading_state::LoadingState::Loaded(());
            //         match result {
            //             Ok(data) => {
            //                 app.state.editors.viewer.columns = data.columns.clone();
            //                 app.state.editors.viewer.rows = data.rows.clone();
            //                 app.state.editors.viewer.status_msg =
            //                     format!("SQL executed: {} rows returned", data.rows.len());
            //             }
            //             Err(e) => {
            //                 app.state.editors.viewer.status_msg = format!("SQL error: {}", e);
            //             }
            //         }
            //         // Process result directly instead of sending message
            //         match result {
            //             Ok(data) => {
            //                 app.state.editors.viewer.columns = data.columns.clone();
            //                 app.state.editors.viewer.rows = data.rows.clone();
            //                 app.state.editors.viewer.status_msg =
            //                     format!("SQL executed: {} rows returned", data.rows.len());
            //             }
            //             Err(e) => {
            //                 app.state.editors.viewer.status_msg = format!("SQL error: {}", e);
            //             }
            //         }
            //         crate::message::Message::Viewer(ViewerMessage::DataLoaded(Ok(app
            //             .state
            //             .editors.viewer
            //             .rows
            //             .clone())))
            //     },
            // );
        }
        ViewerMessage::ExportCsv => {
            // Export data to CSV
            if app.state.editors.viewer.rows.is_empty() {
                app.state.editors.viewer.status_msg = "No data to export".into();
                return Task::none();
            }

            let cols = app.state.editors.viewer.columns.clone();
            let rows = app.state.editors.viewer.rows.clone();
            app.state.editors.viewer.status_msg = "Exporting to CSV...".into();

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
                Ok(p) => app.state.editors.viewer.status_msg = format!("✔ CSV exported to {}", p),
                Err(e) => {
                    app.state.editors.viewer.status_msg = format!("✖ CSV export failed: {}", e);
                }
            }
            Task::none()
        }
        ViewerMessage::RevertEdits => {
            // Revert all unsaved edits
            app.state.editors.viewer.pending_edits.clear();
            app.state.editors.viewer.editing_cell = None;
            app.state.editors.viewer.status_msg = "Reverted all pending edits.".into();
            Task::none()
        }
    }
}
