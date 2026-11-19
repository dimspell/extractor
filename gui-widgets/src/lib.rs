//! Reusable custom Iced widgets for the Dispel GUI and the hex editor.
//!
//! # Public surface
//!
//! `components/mod.rs` is the **single source of truth** for the widget API.
//! It declares every widget module and re-exports their primary types, so both
//! `gui_widgets::TableWidget` and `gui_widgets::components::TableWidget` resolve
//! to the same item. Do not maintain a second, hand-written list here — the
//! one glob below is the only re-export at the crate root.
//!
//! The `core` module is a separate, deliberate facade over the forked iced
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

// Single crate-root facade; the authoritative list lives in components/mod.rs.
pub use components::*;
