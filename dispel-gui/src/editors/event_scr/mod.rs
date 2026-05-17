// event_scr editor module — Custom multi-section editor for EventScript files

pub mod act_tree;
pub mod functions;
pub mod message;
pub mod state;
pub mod update;
pub mod view;

// Re-exports
pub use act_tree::ScriptNode;
pub use functions::EventScriptFunctionIndex;
pub use message::{EventScrEditorMessage, KeyboardShortcut};
pub use state::{EventScriptEditorState, FunctionIndexState};
pub use update::handle;
pub use view::view;
