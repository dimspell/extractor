//! Save.ifo editor: save-slot overview with swapping plus global tail fields.
//!
//! Slot rows are display-only; the editable surface is the global tail block.
//! Swaps are confirmed via a modal and persisted immediately (non-undoable).

mod message;
mod state;
mod update;
mod view;

pub use message::*;
pub use state::*;
pub use update::*;
pub use view::view;
