// Hex editor — universal fallback editor for any binary file the dedicated
// editors don't claim. See plan/Phase 6a in
// /Users/piotr/.claude/plans/could-you-read-the-modular-valley.md.

pub mod inspector;
mod message;
mod provider;
pub mod selection;
mod state;
mod update;
mod view;

pub use message::HexEditorMessage;
pub use provider::{BufferProvider, HexProvider};
pub use selection::Selection;
pub use state::{HexEditorState, DEFAULT_BYTES_PER_ROW};
pub use update::handle;
pub use view::view;
