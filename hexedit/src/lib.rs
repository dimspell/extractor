//! A standalone, embeddable hex editor widget for Iced.
//!
//! # Usage
//!
//! ```rust,ignore
//! use hexedit::{HexEditorState, HexEditorConfig, HexEditorMessage};
//! use hexedit::{update, view};
//!
//! // In your app's update:
//! update(&mut state, &config, msg);
//!
//! // In your app's view:
//! view(&state, &config);
//! ```

pub mod coloring;
pub mod config;
pub mod editing;
pub mod goto;
pub mod inspector;
pub mod layout;
pub mod lua_engine;
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
pub use config::{HexEditorConfig, OnSaveFn};
pub use editing::{EditState, InspectorEditState};
pub use lua_engine::LuaScriptEngine;
pub use message::HexEditorMessage;
pub use pattern::Pattern;
pub use provider::{BufferProvider, HexProvider};
pub use search::{SearchMode, SearchState};
pub use selection::{NavDir, Selection};
pub use state::{HexEditorState, DEFAULT_BYTES_PER_ROW};
pub use update::update;
pub use view::view;
