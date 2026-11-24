//! Reusable custom Iced widgets for the Dispel GUI and the hex editor.
//!
//! # Public surface
//!
//! This file is the **single public API surface** of `gui-widgets`. Every
//! intended public item is re-exported once, here, from the widget
//! implementation modules declared in [`components`]. Both
//! `gui_widgets::TableWidget` and `gui_widgets::components::table_widget::TableWidget`
//! resolve to the same item; the root re-exports are the documented, supported
//! path and are covered by the compile-time API test at the bottom of this file.
//!
//! Widget *modules* stay addressable under [`components`] (for example
//! `gui_widgets::components::toast::Status`), which is useful for the few
//! widget-specific helper types that are not part of the root surface. Do not
//! add a second re-export list there — add the item to the block below instead.
//!
//! The [`core`] module is a separate, deliberate facade over the forked iced
//! internals needed by the advanced (Sweeten) widgets.

/// Iced core types used by Sweeten's advanced widgets.
pub mod core {
    pub use iced::advanced::graphics::core::*;
}

pub mod components;
pub mod lucide;
pub mod sweeten {
    pub mod list;
}
pub mod style;

// ── The one authoritative public re-export list ────────────────────────────
pub use components::collapsible_tree::{CollapsibleTree, RenderContext, TreeNode};
pub use components::context_menu::{ContextMenu, Entry};
pub use components::modal::{Modal, modal};
pub use components::paragraph_cache::ParagraphCache;
pub use components::popover::{Popover, popover};
pub use components::tab_bar::{Tab, TabBar, TabBarEvent};
pub use components::table_widget::{RowFlags, TableColumn, TableState, TableWidget};
pub use components::text_area::{TextAreaContent, textarea, textarea_fill, textarea_style};
pub use components::toast::{
    DEFAULT_TIMEOUT, Manager as ToastManager, Status as ToastStatus, Toast,
};

#[cfg(test)]
mod api_tests;
