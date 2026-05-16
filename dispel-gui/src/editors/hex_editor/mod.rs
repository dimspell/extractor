// Hex editor — universal fallback editor for any binary file the dedicated
// editors don't claim. See plan/Phase 6a in
// /Users/piotr/.claude/plans/could-you-read-the-modular-valley.md.

pub mod coloring;
pub mod editing;
pub mod goto;
pub mod inspector;
pub mod layout;
mod message;
pub mod pattern;
pub mod provider;
pub mod search;
pub mod selection;
mod state;
mod update;
pub mod vanilla_diff;
mod view;

pub use coloring::CellColorProvider;
pub use editing::{EditState, InspectorEditState};
pub use message::HexEditorMessage;
pub use pattern::Pattern;
pub use provider::{BufferProvider, HexProvider};
pub use search::SearchState;
pub use selection::Selection;
pub use state::{HexEditorState, DEFAULT_BYTES_PER_ROW};
pub use update::handle;
pub use view::view;
