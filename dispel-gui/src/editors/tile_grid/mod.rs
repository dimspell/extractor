mod canvas;
mod component;
pub mod message;
pub mod state;
mod update;
pub mod view;

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
pub fn handle_canvas_message(
    msg: TileGridCanvasMessage,
    app: &mut crate::app::App,
) -> iced::Task<crate::message::Message> {
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
