//! The single public widget surface of `gui-widgets`.
//!
//! `lib.rs` re-exports this module wholesale, so this is the one place that
//! declares widget modules and re-exports their primary types. Keep the list
//! here rather than duplicating it at the crate root.

pub mod collapsible_tree;
pub mod context_menu;
pub mod modal;
pub mod paragraph_cache;
pub mod popover;
pub mod tab_bar;
pub mod table_widget;
pub mod text_area;
pub mod toast;

pub use collapsible_tree::{CollapsibleTree, RenderContext, TreeNode};
pub use context_menu::ContextMenu;
pub use modal::modal;
pub use paragraph_cache::ParagraphCache;
pub use popover::popover;
pub use table_widget::{RowFlags, TableColumn, TableState, TableWidget};
pub use text_area::{TextAreaContent, textarea, textarea_fill, textarea_style};
