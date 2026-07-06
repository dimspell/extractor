//! Shared Lucide icon utilities for the GUI.
//!
//! Provides a canonical [`LUCIDE_FONT`] constant and helper functions for
//! rendering Lucide icons via Iced's high-level `text()` widget.

use iced::Font;
use lucide_icons::Icon;

/// Font reference for Lucide icon glyphs.
///
/// Use this with `iced::widget::text(char::from(Icon::X)).font(LUCIDE_FONT)`
/// to render Lucide icons in any Iced text widget.
pub const LUCIDE_FONT: Font = Font::new("lucide");

/// Convert a Lucide [`Icon`] to a `char` suitable for rendering with [`LUCIDE_FONT`].
#[inline]
pub fn icon_char(icon: Icon) -> char {
    char::from(icon)
}
