mod canvas;
mod component;
pub mod message;
pub mod state;
mod update;
pub mod view;

#[cfg(test)]
mod tests;

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
/// Records `EditSelected` mutations in the standard editor's `EditHistory`
/// so undo/redo works for canvas edits.
pub fn handle_canvas_message(
    msg: TileGridCanvasMessage,
    app: &mut crate::app::App,
) -> iced::Task<crate::message::Message> {
    // Step 1: capture everything we need from `app` before the mutable
    // borrow (catalog mutation + history push can't overlap).
    let (old_word, new_desc_for_history, selected_idx) = {
        let view = &app.state.editors.tile_grid_view;
        let catalog = &app.state.editors.tile_grid_editor.state.catalog;
        let old = if let TileGridCanvasMessage::EditSelected(_) = &msg {
            catalog
                .as_ref()
                .and_then(|c| c.first())
                .and_then(|g| view.selected_cell.and_then(|idx| g.data.get(idx as usize).copied()))
        } else {
            None
        };
        let new_desc = if let TileGridCanvasMessage::EditSelected(d) = &msg {
            Some(*d)
        } else {
            None
        };
        (old, new_desc, view.selected_cell)
    };

    // Step 2: mutate the catalog (mutable borrow of app).
    let view = &mut app.state.editors.tile_grid_view;
    let status = if let Some(catalog) = app.state.editors.tile_grid_editor.state.catalog.as_mut() {
        if let Some(grid) = catalog.first_mut() {
            update::handle(msg, view, grid)
        } else {
            None
        }
    } else {
        None
    };

    // Step 3: push to EditHistory (mutable borrow, now safe since the
    // catalog borrow from step 2 is dropped).
    if let (Some(old), Some(new_desc)) = (old_word, new_desc_for_history) {
            let new_word = update::encode_descriptor(new_desc);
            if new_word != old {
                app.state
                    .editors
                    .tile_grid_editor
                    .state
                    .edit_history
                    .push(crate::components::edit_history::EditAction::FieldChange {
                        record_idx: selected_idx.unwrap_or(0) as usize,
                        field: format!("cell_{}", selected_idx.unwrap_or(0)),
                        old_value: old.to_string(),
                        new_value: new_word.to_string(),
                    });
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
