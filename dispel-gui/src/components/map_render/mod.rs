pub mod decode;
pub mod draw_helpers;
pub mod geometry;
pub mod input;
pub mod render_tiles;
pub mod state;
pub mod traits;
pub mod view_state;

pub use decode::decode_tileset_file;
pub use draw_helpers::{diamond_path, draw_item_color};
pub use geometry::{is_visible, screen_to_tile, tile_center, tile_to_screen, tile_world_center};
pub use input::handle_input;
pub use render_tiles::GenericTilesLayer;
pub use state::{EntitySpriteHandle, InternalSpriteHandle, MapCanvasState};
pub use view_state::MapViewState;

/// Rendered width of one isometric tile in world pixels.
pub const TILE_W: f32 = 62.0;
/// Rendered height of one isometric tile in world pixels.
pub const TILE_H: f32 = 32.0;
pub(crate) const HOVER_RADIUS_PX: f32 = 40.0;
