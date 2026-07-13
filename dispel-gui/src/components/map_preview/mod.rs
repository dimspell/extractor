//! Reusable read-only isometric map preview component.
//!
//! Used by the save file viewer to display visited maps with entity overlays.
//! Contains its own simplified canvas (no full sprite decoding, no collision/event
//! editing overlay) and reuses only the dispel-core coordinate math.

pub(crate) mod canvas;
pub(crate) mod message;
pub(crate) mod state;
pub(crate) mod update;
pub(crate) mod view;

pub use message::PreviewMessage;
pub use state::MapPreviewState;
pub use update::handle;
pub use view::view as view_preview;
