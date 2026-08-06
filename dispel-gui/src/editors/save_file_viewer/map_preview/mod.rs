//! Reusable read-only isometric map preview component.
//!
//! Used by the save file viewer to display visited maps with entity overlays.
//! Reuses the shared map_render module for tile rendering and input handling.

pub(crate) mod message;
pub(crate) mod overlay;
pub(crate) mod state;
pub(crate) mod update;
pub(crate) mod view;

pub use message::PreviewMessage;
pub use state::MapPreviewState;
pub use update::handle;
pub use view::view as view_preview;
