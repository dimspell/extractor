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

pub mod decode;
mod draw_helpers;
mod geometry;
pub mod hit_test;
mod input;
mod render_overlays;
mod render_tiles;
mod state;

// ── Re-exports for external consumers ─────────────────────────────────────────

pub use decode::decode_tileset_file;
pub use hit_test::find_hovered_element;
pub use render_overlays::MapCanvasOverlaysLayer;
pub use render_tiles::MapCanvasTilesLayer;

// ── Tile geometry constants ───────────────────────────────────────────────────

/// Rendered width of one tile in pixels (isometric diamond).
pub const TILE_W: f32 = 62.0;
/// Rendered height of one tile in pixels (isometric diamond).
pub const TILE_H: f32 = 32.0;

/// Pixel-space hover radius (world pixels). Entity is considered hovered when
/// the cursor world position is within this many pixels of the tile centre.
pub(crate) const HOVER_RADIUS_PX: f32 = 40.0;

#[cfg(test)]
mod tests;
