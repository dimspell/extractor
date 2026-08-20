//! Read-only save file viewer.
//!
//! Opens `.sav` files and displays parsed game state through sectioned tabs:
//! Overview, Maps, Saved Viewport, Stats, Inventory, Identity, Events, Journal, Raw.
//!
//! **Read-only.** No edit buffers, no undo/redo, no save wiring.

pub use message::SaveFileViewerMessage;
pub use message::{RawHexEditorData, SaveFileLoaded};
pub use state::SaveFileViewerState;
pub use update::handle;
pub use view::view;

pub(crate) mod character;
pub(crate) mod events;
pub(crate) mod helpers;
pub(crate) mod inventory;
pub(crate) mod journal;
pub(crate) mod map_preview;
pub(crate) mod maps;
pub(crate) mod message;
pub(crate) mod overview;
pub(crate) mod party_members;
pub(crate) mod raw;
pub(crate) mod saved_viewport;
pub(crate) mod state;
pub(crate) mod stats;
pub(crate) mod update;
pub(crate) mod view;
