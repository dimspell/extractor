pub mod button;
pub mod context_menu;
pub mod event_script;
pub mod layout;
pub mod progress_bar;
pub mod spreadsheet;
pub mod text;

pub use button::*;
pub use context_menu::*;
pub use event_script::*;
pub use layout::*;
pub use progress_bar::*;
pub use spreadsheet::*;
pub use text::*;

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn test_unused_style_helpers_stay_removed() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/style");
        let removed = [
            ("layout.rs", "log_container"),
            ("layout.rs", "sql_editor_container"),
            ("layout.rs", "normal_mode_chip"),
            ("layout.rs", "edit_mode_chip"),
            ("spreadsheet.rs", "grid_cell"),
            ("spreadsheet.rs", "grid_cell_dirty"),
            ("spreadsheet.rs", "grid_row"),
            ("spreadsheet.rs", "grid_row_even"),
            ("spreadsheet.rs", "resize_handle"),
            ("spreadsheet.rs", "spreadsheet_cell"),
            ("spreadsheet.rs", "spreadsheet_id_cell"),
            ("spreadsheet.rs", "spreadsheet_id_cell_selected"),
            ("spreadsheet.rs", "spreadsheet_cell_editor"),
            ("spreadsheet.rs", "spreadsheet_cell_invalid"),
            ("button.rs", "grid_header_button"),
            ("button.rs", "grid_cell_button"),
            ("button.rs", "pane_header_button"),
            ("button.rs", "spreadsheet_header_button"),
            ("button.rs", "spreadsheet_row"),
            ("button.rs", "spreadsheet_cell_btn"),
            ("button.rs", "tree_menu_button"),
            ("progress_bar.rs", "loading_progress_bar"),
            ("progress_bar.rs", "secondary_progress_bar"),
            ("text.rs", "progress_text_style"),
            ("text.rs", "pane_title_focused"),
            ("text.rs", "pane_title_unfocused"),
            ("text.rs", "normal_mode_text"),
            ("text.rs", "edit_mode_text"),
        ];
        for (file, name) in removed {
            let path = root.join(file);
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
            assert!(
                !source.contains(&format!("pub fn {name}")),
                "{file} must not redefine unused helper {name}"
            );
        }
        assert!(
            !root.join("pane_grid.rs").exists(),
            "the unused pane_grid style module must stay removed"
        );
    }
}
