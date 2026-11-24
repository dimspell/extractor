mod canvas;
mod component;
pub mod message;
pub mod state;
mod update;
pub mod view;

#[cfg(test)]
mod tests;

use crate::components::editable::EditableRecord;

pub use message::TileGridCanvasMessage;
pub use state::TileGridViewState;

/// Handle a `TileGridCanvasMessage`. Mutates the view state and the active
/// grid record (if loaded), and returns a status string for the inspector.
pub fn handle_canvas(
    msg: TileGridCanvasMessage,
    view: &mut TileGridViewState,
    grid: &mut dispel_core::TileGrid,
) -> Option<String> {
    update::handle(msg, view, grid)
}

/// Handle a `TileGridCanvasMessage` from the dispatch table.
/// Mutates the registry's `tile_grid_view` and the first loaded grid record.
/// Canvas and inspector edits are recorded as a single `FieldChange` on the
/// grid's `data` field so the standard editor's undo/redo restores the exact
/// words via the `VecU32` `EditableRecord` round-trip.
pub fn handle_canvas_message(
    msg: TileGridCanvasMessage,
    app: &mut crate::app::App,
) -> iced::Task<crate::message::Message> {
    // A `.dat` grid is a single record whose `data` vector holds every cell.
    // Snapshot that field before the mutation so history stores whole-vector
    // before/after values (a `cell_N` field does not exist on `TileGrid`).
    let is_edit = matches!(
        msg,
        TileGridCanvasMessage::EditSelected(_) | TileGridCanvasMessage::CommitFieldBuffer { .. }
    );
    let old_data = if is_edit {
        app.state
            .editors
            .tile_grid_editor
            .state
            .catalog
            .as_ref()
            .and_then(|catalog| catalog.first())
            .map(|grid| grid.get_field("data"))
    } else {
        None
    };

    let view = &mut app.state.editors.tile_grid_view;
    let status = if let Some(catalog) = app.state.editors.tile_grid_editor.state.catalog.as_mut()
        && let Some(grid) = catalog.first_mut()
    {
        update::handle(msg, view, grid)
    } else {
        None
    };

    if let Some(old_data) = old_data {
        let new_data = app
            .state
            .editors
            .tile_grid_editor
            .state
            .catalog
            .as_ref()
            .and_then(|catalog| catalog.first())
            .map(|grid| grid.get_field("data"));
        if let Some(new_data) = new_data
            && new_data != old_data
        {
            app.state.editors.tile_grid_editor.state.edit_history.push(
                crate::components::edit_history::EditAction::FieldChange {
                    record_idx: 0,
                    field: "data".to_string(),
                    old_value: old_data,
                    new_value: new_data,
                },
            );
            app.state.workspace.mark_modified();
        }
    }

    if let Some(s) = status {
        app.state.status_msg = s;
    }
    iced::Task::none()
}

crate::define_standard_editor! {
    name: tile_grid,
    name_pascal: TileGrid,
    record: dispel_core::TileGrid,
    field: tile_grid_editor,
    file: "Map/Multi/*.dat",
}
