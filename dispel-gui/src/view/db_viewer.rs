use crate::app::App;
use crate::db_viewer_state::PAGE_SIZE;
use crate::message::Message;
use crate::style;
use iced::widget::{
    button, column, container, horizontal_space, row, scrollable, text, text_input, vertical_space,
};
use iced::{Element, Fill, Font};

impl App {
    pub fn view_db_viewer(&self) -> Element<'_, Message> {
        let v = &self.viewer;

        // ── Connection toolbar ──
        let conn_row = container(
            row![
                text_input("database.sqlite", &v.db_path)
                    .on_input(Message::ViewerDbPathChanged)
                    .padding(8)
                    .size(13),
                button(text("…").size(12))
                    .padding([6, 12])
                    .on_press(Message::ViewerBrowseDb)
                    .style(style::browse_button),
                button(text("Connect").size(13))
                    .padding([8, 16])
                    .on_press(Message::ViewerConnect)
                    .style(style::run_button),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .padding(10),
        )
        .width(Fill)
        .style(style::toolbar_container);

        // ── Table selector chips ──
        let table_chips: Vec<Element<Message>> = v
            .tables
            .iter()
            .map(|t| {
                let is_active = v.active_table.as_deref() == Some(t.as_str());
                let btn = button(text(t).size(11))
                    .padding([4, 10])
                    .on_press(Message::ViewerSelectTable(t.clone()));
                if is_active {
                    btn.style(style::active_chip)
                } else {
                    btn.style(style::chip)
                }
                .into()
            })
            .collect();
        let table_row = if table_chips.is_empty() {
            container(
                text("Connect to a database to see tables.")
                    .size(12)
                    .style(style::subtle_text),
            )
            .padding(10)
        } else {
            container(scrollable(row(table_chips).spacing(4).padding(6).wrap()).height(60))
                .padding([4, 10])
        };

        // ── Action toolbar ──
        let search_input = text_input("🔍 Search all columns…", &v.search)
            .on_input(Message::ViewerSearch)
            .padding(8)
            .size(12)
            .width(250);
        let sql_toggle = button(text(if v.sql_mode { "Hide SQL" } else { "SQL Editor" }).size(11))
            .padding([5, 10])
            .on_press(Message::ViewerToggleSql)
            .style(style::chip);
        let export_btn = button(text("📥 Export CSV").size(11))
            .padding([5, 10])
            .on_press(Message::ViewerExportCsv)
            .style(style::chip);

        let edit_count = v.pending_edits.len();
        let commit_btn = if edit_count > 0 {
            button(text(format!("💾 Commit ({edit_count})")).size(11))
                .padding([5, 10])
                .on_press(Message::ViewerCommit)
                .style(style::commit_button)
        } else {
            button(text("💾 Commit").size(11))
                .padding([5, 10])
                .style(style::run_button_disabled)
        };
        let revert_btn = if edit_count > 0 {
            button(text("↩ Revert").size(11))
                .padding([5, 10])
                .on_press(Message::ViewerRevertEdits)
                .style(style::chip)
        } else {
            button(text("↩ Revert").size(11))
                .padding([5, 10])
                .style(style::run_button_disabled)
        };

        let action_row = container(
            row![
                search_input,
                horizontal_space(),
                sql_toggle,
                export_btn,
                revert_btn,
                commit_btn
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center)
            .padding(8),
        )
        .width(Fill)
        .style(style::toolbar_container);

        // ── SQL editor (collapsible) ──
        let sql_area: Element<Message> = if v.sql_mode {
            let sql_input = text_input("SELECT * FROM ...", &v.sql_query)
                .on_input(Message::ViewerSqlChanged)
                .padding(10)
                .size(13)
                .font(Font::MONOSPACE);
            let run_btn = button(text("▶ Run").size(12))
                .padding([6, 14])
                .on_press(Message::ViewerRunSql)
                .style(style::run_button);
            container(
                row![sql_input, run_btn]
                    .spacing(8)
                    .align_y(iced::Alignment::Center)
                    .padding(8),
            )
            .width(Fill)
            .style(style::sql_editor_container)
            .into()
        } else {
            vertical_space().height(0).into()
        };

        // ── Data grid ──
        let grid = self.view_grid();

        // ── Pagination ──
        let max_page = if v.total_rows == 0 {
            0
        } else {
            v.total_rows.saturating_sub(1) / PAGE_SIZE
        };
        let prev_btn = if v.page > 0 {
            button(text("◀ Prev").size(11))
                .padding([4, 10])
                .on_press(Message::ViewerPrevPage)
                .style(style::chip)
        } else {
            button(text("◀ Prev").size(11))
                .padding([4, 10])
                .style(style::run_button_disabled)
        };
        let next_btn = if v.page < max_page {
            button(text("Next ▶").size(11))
                .padding([4, 10])
                .on_press(Message::ViewerNextPage)
                .style(style::chip)
        } else {
            button(text("Next ▶").size(11))
                .padding([4, 10])
                .style(style::run_button_disabled)
        };
        let page_info = text(format!("Page {} / {}", v.page + 1, max_page + 1)).size(11);

        let status_row = container(
            row![
                text(&v.status_msg).size(11).style(style::subtle_text),
                horizontal_space(),
                prev_btn,
                page_info,
                next_btn
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .padding([6, 12]),
        )
        .width(Fill)
        .style(style::status_bar);

        column![conn_row, table_row, action_row, sql_area, grid, status_row]
            .spacing(0)
            .width(Fill)
            .height(Fill)
            .into()
    }

    pub fn view_grid(&self) -> Element<'_, Message> {
        let v = &self.viewer;
        if v.columns.is_empty() {
            return container(
                text("Select a table to view its data.")
                    .size(14)
                    .style(style::subtle_text),
            )
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill)
            .into();
        }

        // ── Header row ──
        let header_cells: Vec<Element<Message>> = v
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                let sort_indicator = if v.sort_col == Some(i) {
                    v.sort_dir.arrow()
                } else {
                    ""
                };
                let label = format!("{}{}", col.name, sort_indicator);
                let pk_marker = if col.is_pk { " 🔑" } else { "" };
                button(
                    text(format!("{label}{pk_marker}"))
                        .size(11)
                        .font(Font::MONOSPACE),
                )
                .width(150)
                .padding([8, 6])
                .on_press(Message::ViewerSortColumn(i))
                .style(style::grid_header_button)
                .into()
            })
            .collect();
        let header = container(row(header_cells).spacing(0)).style(style::grid_header_cell);

        // ── Data rows ──
        let data_rows: Vec<Element<Message>> = v
            .rows
            .iter()
            .enumerate()
            .map(|(ri, row_data)| {
                let cells: Vec<Element<Message>> = row_data
                    .iter()
                    .enumerate()
                    .map(|(ci, cell_val)| {
                        let is_editing = v.editing_cell == Some((ri, ci));
                        let is_dirty = v.pending_edits.contains_key(&(ri, ci));
                        let display_val = v.pending_edits.get(&(ri, ci)).unwrap_or(cell_val);

                        let cell_style = if is_dirty {
                            style::grid_cell_dirty
                        } else if ri % 2 == 0 {
                            style::grid_cell
                        } else {
                            style::grid_cell_even
                        };

                        let inner: Element<Message> = if is_editing {
                            text_input("", &v.edit_buffer)
                                .on_input(Message::ViewerCellEdit)
                                .on_submit(Message::ViewerCellConfirm)
                                .padding(4)
                                .size(11)
                                .font(Font::MONOSPACE)
                                .into()
                        } else {
                            button(text(display_val).size(11).font(Font::MONOSPACE))
                                .width(Fill)
                                .padding([6, 4])
                                .on_press(Message::ViewerCellClick(ri, ci))
                                .style(style::grid_cell_button)
                                .into()
                        };

                        container(inner).width(150).style(cell_style).into()
                    })
                    .collect();
                row(cells).spacing(0).into()
            })
            .collect();

        let grid_content = column![header, column(data_rows).spacing(0)].spacing(0);

        scrollable(grid_content)
            .direction(iced::widget::scrollable::Direction::Both {
                vertical: Default::default(),
                horizontal: Default::default(),
            })
            .height(Fill)
            .width(Fill)
            .into()
    }
}
