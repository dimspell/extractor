// Hex editor — universal fallback editor for any binary file the dedicated
// editors don't claim.

pub mod coloring;
pub mod config;
pub mod dispel_save;
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
pub use config::HexEditorConfig;
pub use editing::{EditState, InspectorEditState};
pub use message::HexEditorMessage;
pub use pattern::Pattern;
pub use provider::{BufferProvider, HexProvider};
pub use search::SearchState;
pub use selection::Selection;
pub use state::{HexEditorState, DEFAULT_BYTES_PER_ROW};
pub use update::update;
pub use view::view;
