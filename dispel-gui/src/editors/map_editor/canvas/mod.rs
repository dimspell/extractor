//! Canvas-related types and rendering for the map editor.
//!
//! Renders isometric tiles from GTL (ground) and BTL (building) layers using
//! decoded image handles stored in `MapEditorState`. Supports mouse drag for
//! panning and scroll wheel for zooming.
//!
//! Three `canvas::Program` implementations work together in a `Stack`:
//! - `MapCanvas` handles all user input (drag, click, scroll, hover)
//! - `MapCanvasTilesLayer` renders tiles + interlaced objects (buildings, sprites, entities)
//! - `MapCanvasOverlaysLayer` renders overlays (collisions, events, selection, cursor)

pub mod hit_test;
mod input;
mod render_overlays;

// ── Re-exports from shared map_render module ─────────────────────────────────

pub use crate::components::map_render::{
    EntitySpriteHandle, GenericTilesLayer, InternalSpriteHandle, MapCanvasState, MapViewState,
    TILE_H, TILE_W, decode_tileset_file, draw_helpers, geometry,
};

// ── Re-exports for external consumers ─────────────────────────────────────────

pub use hit_test::{find_entity_at, find_hovered_element, find_tile_at};
pub use render_overlays::MapCanvasOverlaysLayer;

/// Tile layer type alias — delegates to the shared generic implementation.
pub type MapCanvasTilesLayer<'a> =
    GenericTilesLayer<'a, crate::editors::map_editor::state::MapEditorState>;

#[cfg(test)]
mod tests;
