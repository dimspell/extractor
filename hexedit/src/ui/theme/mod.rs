//! Centralised colour theme for the hex editor.
//!
//! All hardcoded colour literals scattered across the codebase are
//! consolidated into [`HexEditorTheme`] with two built-in variants:
//! [`DARK_THEME`] (the original look) and [`LIGHT_THEME`] (parchment
//! background for accessibility / bright environments).

use std::fmt;

use iced::Color;

/// Which built-in colour theme is currently active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeVariant {
    /// Dark leather/amber theme (the original).
    #[default]
    Dark,
    /// Warm parchment light theme.
    Light,
}

impl ThemeVariant {
    /// All variants, in display order.
    pub const ALL: [ThemeVariant; 2] = [ThemeVariant::Dark, ThemeVariant::Light];

    /// Human-readable label for the variant.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            ThemeVariant::Dark => "Dark",
            ThemeVariant::Light => "Light",
        }
    }

    /// Return the static theme constant for this variant.
    #[must_use]
    pub const fn theme(&self) -> &'static HexEditorTheme {
        match self {
            ThemeVariant::Dark => &DARK_THEME,
            ThemeVariant::Light => &LIGHT_THEME,
        }
    }
}

impl fmt::Display for ThemeVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

// ── Const helper ────────────────────────────────────────────────────────────

/// Parse a hex RGB colour (e.g. `0x14110f`) into an Iced [`Color`].
///
/// This is a `const fn` so the theme constants can be computed at compile time.
#[must_use]
pub const fn hex(c: u32) -> Color {
    Color::from_rgb(
        ((c >> 16) & 0xFF) as f32 / 255.0,
        ((c >> 8) & 0xFF) as f32 / 255.0,
        (c & 0xFF) as f32 / 255.0,
    )
}

// ── Theme definition ────────────────────────────────────────────────────────

/// A complete colour palette for the hex editor.
///
/// **Flat struct, no nesting** — every consumer references a single field
/// so there is never ambiguity about where a colour lives.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HexEditorTheme {
    // ── Hex matrix ──────────────────────────────────────────────────────
    pub matrix_bg: Color,
    pub address_gutter_bg: Color,
    pub address_fg: Color,
    pub hex_fg: Color,
    pub ascii_fg: Color,

    // ── Column header ───────────────────────────────────────────────────
    pub header_bg: Color,
    pub header_fg: Color,
    pub header_separator: Color,

    // ── Group separators ────────────────────────────────────────────────
    pub group_separator: Color,

    // ── Selection / cursor ──────────────────────────────────────────────
    pub selection_bg: Color,
    pub cursor_bg: Color,
    pub selection_fg: Color,
    pub caret: Color,

    // ── Edit / dirty / diff overlays ────────────────────────────────────
    pub dirty_bg: Color,
    pub dirty_fg: Color,
    pub diff_bg: Color,
    pub diff_fg: Color,
    pub edit_bg: Color,
    pub edit_fg: Color,

    // ── Search matches ──────────────────────────────────────────────────
    pub search_current_bg: Color,
    pub search_current_fg: Color,
    pub search_match_bg: Color,
    pub search_match_fg: Color,

    // ── Annotation column ───────────────────────────────────────────────
    pub annotation_fg: Color,
    pub annotation_inactive: Color,
    pub annotation_separator: Color,

    // ── Scrollbar (vertical) ────────────────────────────────────────────
    pub scrollbar_bg: Color,
    pub scrollbar_thumb: Color,
    pub scrollbar_thumb_hover: Color,
    pub scrollbar_search_dot: Color,
    pub scrollbar_cursor_dot: Color,

    // ── Modal dialogs ───────────────────────────────────────────────────
    pub modal_bg: Color,
    pub modal_border: Color,
    pub modal_heading_fg: Color,
    pub modal_muted_fg: Color,
    pub modal_error_fg: Color,

    // ── Export preview ──────────────────────────────────────────────────
    pub export_preview_bg: Color,
    pub export_preview_border: Color,
    pub export_info_fg: Color,

    // ── Search overlay bar ──────────────────────────────────────────────
    pub search_overlay_bg: Color,
    pub search_overlay_border: Color,

    // ── Minimap ─────────────────────────────────────────────────────────
    pub minimap_bg: Color,
    pub minimap_separator: Color,
    pub minimap_cursor_marker: Color,
    pub minimap_dirty_pixel: Color,
    pub minimap_diff_pixel: Color,

    // ── Pattern panel ───────────────────────────────────────────────────
    pub pattern_panel_bg: Color,
    pub pattern_panel_border: Color,
    pub pattern_active_highlight: Color,
    pub pattern_count_fg: Color,

    // ── Statistics panel ────────────────────────────────────────────────
    pub stats_heading_fg: Color,
    pub stats_muted_fg: Color,
    pub stats_bar_padding: Color,
    pub stats_bar_low: Color,
    pub stats_bar_mid_low: Color,
    pub stats_bar_mid_high: Color,
    pub stats_bar_high: Color,
    pub stats_bar_default: Color,
    pub stats_structure_uniform: Color,
    pub stats_structure_high_entropy: Color,
    pub stats_structure_low_entropy: Color,
    pub stats_structure_mixed: Color,

    // ── Byte-colouring schemes ──────────────────────────────────────────
    /// The colour used for null bytes when `dim_nulls` is active.
    pub default_null_dim: Color,
    /// Monochrome scheme default foreground (used for all bytes).
    pub monochrome_fg: Color,
    /// 18-group nybble palette — indices 0..15 mapped by high nybble.
    /// `0x00` gets `default_null_dim`; `0xFF` gets `nybble_ff`.
    pub nybble_palette: [Color; 16],
    pub nybble_ff: Color,
    /// Category scheme colours (in order: whitespace, printable, ctrl, non-ascii).
    pub category_whitespace: Color,
    pub category_printable: Color,
    pub category_ctrl: Color,
    pub category_non_ascii: Color,
    /// HSL params for the rainbow / heatmap gradient schemes.
    /// The lightness value is chosen so every byte meets WCAG AA against
    /// `matrix_bg` (0.70 for dark bg, ~0.35 for light bg).
    pub scheme_saturation: f32,
    pub scheme_lightness: f32,

    // ── Pattern overlay palettes ────────────────────────────────────────
    /// 16 background colours cycled through by pattern index.
    pub pattern_bg_palette: [Color; 16],
    /// 16 foreground (text) colours cycled through by pattern index.
    pub pattern_fg_palette: [Color; 16],

    // ── Iced application palette (standalone hexedit binary) ────────────
    pub iced_bg: Color,
    pub iced_text: Color,
    pub iced_primary: Color,
    pub iced_success: Color,
    pub iced_danger: Color,
    pub iced_warning: Color,
}

// ── Colour scheme helpers ─────────────────────────────────────────────────

impl HexEditorTheme {
    /// Pick the colour for a single byte under the given scheme.
    #[must_use]
    pub fn scheme_color(&self, scheme: super::coloring::ColorScheme, b: u8) -> Color {
        match scheme {
            super::coloring::ColorScheme::Monochrome => self.monochrome_fg,
            super::coloring::ColorScheme::Nybble => self.nybble_color(b),
            super::coloring::ColorScheme::Categories => self.category_color(b),
            super::coloring::ColorScheme::Rainbow => self.rainbow_color(b),
            super::coloring::ColorScheme::Heatmap => self.heatmap_color(b),
        }
    }

    /// Map a byte to a colour based on its high nybble — 18 groups.
    fn nybble_color(&self, b: u8) -> Color {
        match b {
            0x00 => self.default_null_dim,
            0xFF => self.nybble_ff,
            _ => self.nybble_palette[(b >> 4) as usize],
        }
    }

    /// 6 semantic categories matching hexyl's default scheme.
    fn category_color(&self, b: u8) -> Color {
        match b {
            0x00 => self.default_null_dim,
            0x09..=0x0D => self.category_whitespace,
            0x20..=0x7E => self.category_printable,
            0x7F => self.category_ctrl,
            _ if b < 0x20 => self.category_ctrl,
            _ => self.category_non_ascii,
        }
    }

    /// Continuous rainbow hue (0°–300°, skipping red-to-red wrap).
    fn rainbow_color(&self, b: u8) -> Color {
        let hue = (b as f32 / 255.0) * 300.0;
        let (r, g, b_) = hsl_to_rgb(hue, self.scheme_saturation, self.scheme_lightness);
        Color::from_rgb(r, g, b_)
    }

    /// Cold-to-hot heatmap (blue → cyan → green → yellow → red).
    fn heatmap_color(&self, b: u8) -> Color {
        let hue = 240.0 * (1.0 - b as f32 / 255.0);
        let (r, g, b_) = hsl_to_rgb(hue, self.scheme_saturation, self.scheme_lightness);
        Color::from_rgb(r, g, b_)
    }
}

/// Convert HSL to RGB, all components in 0..1 range.
#[must_use]
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r1, g1, b1) = match h as u16 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (r1 + m, g1 + m, b1 + m)
}

// ── Built-in themes ────────────────────────────────────────────────────────

/// Default dark theme — the original hex editor look with a near-black
/// background and warm amber/brown tones.
pub const DARK_THEME: HexEditorTheme = HexEditorTheme {
    // ── Matrix ──────────────────────────────────────────────────────────
    matrix_bg: hex(0x14110f),
    address_gutter_bg: hex(0x14110f),
    address_fg: hex(0x7a6f64),
    hex_fg: hex(0xd4cabd),
    ascii_fg: hex(0xb8a898),

    // ── Header ──────────────────────────────────────────────────────────
    header_bg: hex(0x1a1614),
    header_fg: hex(0x8a7a6a),
    header_separator: hex(0x2a2218),

    // ── Separators ──────────────────────────────────────────────────────
    group_separator: hex(0x251f1a),

    // ── Selection / cursor ──────────────────────────────────────────────
    selection_bg: hex(0x3b2a18),
    cursor_bg: hex(0x6a4a26),
    selection_fg: hex(0xfff4e0),
    caret: hex(0xfff4e0),

    // ── Edit / dirty / diff ─────────────────────────────────────────────
    dirty_bg: hex(0x4a1f1a),
    dirty_fg: hex(0xff9d6e),
    diff_bg: hex(0x232f1f),
    diff_fg: hex(0x9bd07a),
    edit_bg: hex(0xc25e1c),
    edit_fg: hex(0xfff8ee),

    // ── Search ──────────────────────────────────────────────────────────
    search_current_bg: hex(0x4a6a2a),
    search_current_fg: hex(0xfff8ee),
    search_match_bg: hex(0x2a4a2a),
    search_match_fg: hex(0xfff4e0),

    // ── Annotation column ───────────────────────────────────────────────
    annotation_fg: hex(0xd4cabd),
    annotation_inactive: hex(0x6a6050),
    annotation_separator: hex(0x6a6050),

    // ── Scrollbar ───────────────────────────────────────────────────────
    scrollbar_bg: hex(0x141210),
    scrollbar_thumb: hex(0x5d4037),
    scrollbar_thumb_hover: hex(0xB97024),
    scrollbar_search_dot: hex(0x4a7a2a),
    scrollbar_cursor_dot: hex(0xB97024),

    // ── Modal dialogs ───────────────────────────────────────────────────
    modal_bg: hex(0x201b18),
    modal_border: hex(0x4a3f35),
    modal_heading_fg: hex(0xa0907a),
    modal_muted_fg: hex(0x7a6f64),
    modal_error_fg: hex(0xff8a6e),

    // ── Export preview ──────────────────────────────────────────────────
    export_preview_bg: hex(0x15110e),
    export_preview_border: hex(0x3a3026),
    export_info_fg: hex(0x7a6f64),

    // ── Search overlay ──────────────────────────────────────────────────
    search_overlay_bg: hex(0x1e1e1e),
    search_overlay_border: hex(0x3d3d3d),

    // ── Minimap ─────────────────────────────────────────────────────────
    minimap_bg: hex(0x141210),
    minimap_separator: hex(0x2a2218),
    minimap_cursor_marker: hex(0xB97024),
    minimap_dirty_pixel: hex(0x4a1f1a),
    minimap_diff_pixel: hex(0x232f1f),

    // ── Pattern panel ───────────────────────────────────────────────────
    pattern_panel_bg: hex(0x1e1e1e),
    pattern_panel_border: hex(0x3d3d3d),
    pattern_active_highlight: hex(0x3b2a18),
    pattern_count_fg: hex(0x8a7a6a),

    // ── Statistics panel ────────────────────────────────────────────────
    stats_heading_fg: hex(0x8a7a6a),
    stats_muted_fg: hex(0x7a6f64),
    stats_bar_padding: hex(0x6a5a4a),
    stats_bar_low: hex(0x7a8a5a),
    stats_bar_mid_low: hex(0x5a8a7a),
    stats_bar_mid_high: hex(0x8a7a4a),
    stats_bar_high: hex(0xaa5a4a),
    stats_bar_default: hex(0x8a7a6a),
    stats_structure_uniform: hex(0x6a8a5a),
    stats_structure_high_entropy: hex(0xaa4a3a),
    stats_structure_low_entropy: hex(0x5a7a8a),
    stats_structure_mixed: hex(0x8a7a6a),

    // ── Byte-colouring schemes ──────────────────────────────────────────
    default_null_dim: Color::from_rgb(0x4A as f32 / 255.0, 0x43 as f32 / 255.0, 0x39 as f32 / 255.0),
    monochrome_fg: hex(0xd4cabd),
    nybble_palette: [
        hex(0x9AB09A), hex(0xA0B484), hex(0xAAB478), hex(0xB8B078),
        hex(0xC4A878), hex(0xCE9C78), hex(0xD49078), hex(0xCC8880),
        hex(0xC48C98), hex(0xB890B0), hex(0x8E90AC), hex(0x7EA0AC),
        hex(0x7AAC98), hex(0x84AC84), hex(0xA8A088), hex(0xB2A690),
    ],
    nybble_ff: hex(0xd4cabd),
    category_whitespace: hex(0x618950),
    category_printable: hex(0xb8a898),
    category_ctrl: hex(0x967962),
    category_non_ascii: hex(0xa26f8f),
    scheme_saturation: 0.85,
    scheme_lightness: 0.70,

    // ── Pattern palettes ────────────────────────────────────────────────
    pattern_bg_palette: [
        hex(0x1a3a4f), hex(0x4f2e1a), hex(0x1a4f2e), hex(0x3b1a4f),
        hex(0x4f4a1a), hex(0x2e1a4f), hex(0x4f1a1a), hex(0x1a3b3b),
        hex(0x3b2e1a), hex(0x2e4f1a), hex(0x4f2e3b), hex(0x1a4f4f),
        hex(0x4f251a), hex(0x1a3b25), hex(0x3b3b1a), hex(0x251a4f),
    ],
    pattern_fg_palette: [
        hex(0x6ab0d0), hex(0xd08a6a), hex(0x6ad08a), hex(0xa06ad0),
        hex(0xd0cb6a), hex(0x8a6ad0), hex(0xd06a6a), hex(0x6ad0d0),
        hex(0xd0af6a), hex(0x8ad06a), hex(0xd06a9a), hex(0x6ad0af),
        hex(0xd0856a), hex(0x6ad085), hex(0xafd06a), hex(0x856ad0),
    ],

    // ── Iced application palette ────────────────────────────────────────
    iced_bg: hex(0x2a2a2a),
    iced_text: hex(0xeae0c8),
    iced_primary: hex(0x8b5a2b),
    iced_success: hex(0x2d5a27),
    iced_danger: hex(0x800000),
    iced_warning: hex(0x8b8b00),
};

/// Light theme — warm parchment background for accessibility.
///
/// Designed for WCAG AA (4.5:1) contrast against `#F5EDE0` (the matrix
/// background) whenever possible.
pub const LIGHT_THEME: HexEditorTheme = HexEditorTheme {
    // ── Matrix ──────────────────────────────────────────────────────────
    matrix_bg: hex(0xF5EDE0),
    address_gutter_bg: hex(0xEBE2D3),
    address_fg: hex(0x604E3E),
    hex_fg: hex(0x3A3228),
    ascii_fg: hex(0x3A3228),

    // ── Header ──────────────────────────────────────────────────────────
    header_bg: hex(0xE0D4C2),
    header_fg: hex(0x4A3F30),
    header_separator: hex(0xC0B4A2),

    // ── Separators ──────────────────────────────────────────────────────
    group_separator: hex(0xD0C4B2),

    // ── Selection / cursor ──────────────────────────────────────────────
    selection_bg: hex(0xBBD8FF),
    cursor_bg: hex(0x64A6F0),
    selection_fg: hex(0x1A1A1A),
    caret: hex(0x2563EB),

    // ── Edit / dirty / diff ─────────────────────────────────────────────
    dirty_bg: hex(0xFECACA),
    dirty_fg: hex(0x7F1D1D),
    diff_bg: hex(0xBBF7D0),
    diff_fg: hex(0x166534),
    edit_bg: hex(0x93C5FD),
    edit_fg: hex(0x1E3A5F),

    // ── Search ──────────────────────────────────────────────────────────
    search_current_bg: hex(0x86EFAC),
    search_current_fg: hex(0x14532D),
    search_match_bg: hex(0xA7F3D0),
    search_match_fg: hex(0x064E3B),

    // ── Annotation column ───────────────────────────────────────────────
    annotation_fg: hex(0x3A3228),
    annotation_inactive: hex(0xA09080),
    annotation_separator: hex(0xC0B4A2),

    // ── Scrollbar ───────────────────────────────────────────────────────
    scrollbar_bg: hex(0xE0D4C2),
    scrollbar_thumb: hex(0xA09080),
    scrollbar_thumb_hover: hex(0x7A6A5A),
    scrollbar_search_dot: hex(0x16A34A),
    scrollbar_cursor_dot: hex(0x2563EB),

    // ── Modal dialogs ───────────────────────────────────────────────────
    modal_bg: hex(0xFFFFFF),
    modal_border: hex(0xC0B4A2),
    modal_heading_fg: hex(0x2A2218),
    modal_muted_fg: hex(0x706050),
    modal_error_fg: hex(0xDC2626),

    // ── Export preview ──────────────────────────────────────────────────
    export_preview_bg: hex(0xEBE2D3),
    export_preview_border: hex(0xC0B4A2),
    export_info_fg: hex(0x706050),

    // ── Search overlay ──────────────────────────────────────────────────
    search_overlay_bg: hex(0xF5EDE0),
    search_overlay_border: hex(0xC0B4A2),

    // ── Minimap ─────────────────────────────────────────────────────────
    minimap_bg: hex(0xE0D4C2),
    minimap_separator: hex(0xC0B4A2),
    minimap_cursor_marker: hex(0x2563EB),
    minimap_dirty_pixel: hex(0xFECACA),
    minimap_diff_pixel: hex(0xBBF7D0),

    // ── Pattern panel ───────────────────────────────────────────────────
    pattern_panel_bg: hex(0xF5EDE0),
    pattern_panel_border: hex(0xC0B4A2),
    pattern_active_highlight: hex(0xBBD8FF),
    pattern_count_fg: hex(0x4A3F30),

    // ── Statistics panel ────────────────────────────────────────────────
    stats_heading_fg: hex(0x4A3F30),
    stats_muted_fg: hex(0x706050),
    stats_bar_padding: hex(0xC0B4A2),
    stats_bar_low: hex(0x86EFAC),
    stats_bar_mid_low: hex(0xFDE68A),
    stats_bar_mid_high: hex(0xFDBA74),
    stats_bar_high: hex(0xFCA5A5),
    stats_bar_default: hex(0xA09080),
    stats_structure_uniform: hex(0x3B82F6),
    stats_structure_high_entropy: hex(0xDC2626),
    stats_structure_low_entropy: hex(0x16A34A),
    stats_structure_mixed: hex(0xCA8A04),

    // ── Byte-colouring schemes ──────────────────────────────────────────
    default_null_dim: hex(0xA09080),
    monochrome_fg: hex(0x3A3228),
    nybble_palette: [
        hex(0x887c6f), hex(0x7a8f5a), hex(0x6a8f4a), hex(0x5a8f5a),
        hex(0x4a8f6a), hex(0x5e856f), hex(0x6f855e), hex(0x8a7d54),
        hex(0xa07346), hex(0xb36a46), hex(0xbb644c), hex(0xc35e68),
        hex(0xb26a74), hex(0xa0726d), hex(0x897c65), hex(0x7a8f5a),
    ],
    nybble_ff: hex(0x3A3228),
    category_whitespace: hex(0x166534),
    category_printable: hex(0x3A3228),
    category_ctrl: hex(0x92400E),
    category_non_ascii: hex(0x7C3AED),
    scheme_saturation: 0.85,
    scheme_lightness: 0.35,

    // ── Pattern palettes ────────────────────────────────────────────────
    pattern_bg_palette: [
        hex(0x93C5FD), hex(0xFCA5A5), hex(0x86EFAC), hex(0xC4B5FD),
        hex(0xFDE68A), hex(0xA5B4FC), hex(0xFECACA), hex(0x99F6E4),
        hex(0xFDBA74), hex(0xBBF7D0), hex(0xF9A8D4), hex(0x67E8F9),
        hex(0xFBBF24), hex(0x6EE7B7), hex(0xFCD34D), hex(0xDDD6FE),
    ],
    pattern_fg_palette: [
        hex(0x1E3A5F), hex(0x7F1D1D), hex(0x166534), hex(0x3B0764),
        hex(0x713F12), hex(0x312E81), hex(0x7F1D1D), hex(0x0F766E),
        hex(0x7C2D12), hex(0x14532D), hex(0x831843), hex(0x164E63),
        hex(0x78350F), hex(0x134E4A), hex(0x713F12), hex(0x4A1D96),
    ],

    // ── Iced application palette ────────────────────────────────────────
    iced_bg: hex(0xF5EDE0),
    iced_text: hex(0x2A2218),
    iced_primary: hex(0x8B5A2B),
    iced_success: hex(0x16A34A),
    iced_danger: hex(0xDC2626),
    iced_warning: hex(0xCA8A04),
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::coloring::{contrast_ratio, relative_luminance};

    // ── WCAG AA helpers ─────────────────────────────────────────────────────

    /// Assert that two colours meet a given contrast threshold.
    fn assert_contrast(label: &str, fg: Color, bg: Color, threshold: f32) {
        let cr = contrast_ratio(&fg, &bg);
        assert!(
            cr >= threshold,
            "WCAG AA FAIL [{}]: fg=({:.3},{:.3},{:.3}) bg=({:.3},{:.3},{:.3}) \
             CR={:.2} < {:.1}",
            label, fg.r, fg.g, fg.b, bg.r, bg.g, bg.b, cr, threshold,
        );
    }

    /// Convenience: WCAG AA normal text threshold (4.5:1).
    fn check(label: &str, fg: Color, bg: Color) {
        assert_contrast(label, fg, bg, 4.5);
    }

    /// Convenience: WCAG AA non-text / large-text threshold (3:1).
    fn check_ui(label: &str, fg: Color, bg: Color) {
        assert_contrast(label, fg, bg, 3.0);
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  Colour-preservation tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn dark_theme_default_null_dim_matches_original() {
        // The original DEFAULT_NULL_DIM was Color::from_rgb(74/255, 67/255, 57/255).
        let expected = Color::from_rgb(74.0 / 255.0, 67.0 / 255.0, 57.0 / 255.0);
        let (dr, dg, db) = (
            DARK_THEME.default_null_dim.r,
            DARK_THEME.default_null_dim.g,
            DARK_THEME.default_null_dim.b,
        );
        assert!((dr - expected.r).abs() < 0.001);
        assert!((dg - expected.g).abs() < 0.001);
        assert!((db - expected.b).abs() < 0.001);
    }

    #[test]
    fn dark_theme_matrix_bg_matches_original() {
        let expected = Color::from_rgb(20.0 / 255.0, 17.0 / 255.0, 15.0 / 255.0);
        assert!((DARK_THEME.matrix_bg.r - expected.r).abs() < 0.001);
        assert!((DARK_THEME.matrix_bg.g - expected.g).abs() < 0.001);
        assert!((DARK_THEME.matrix_bg.b - expected.b).abs() < 0.001);
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  Structural palette tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn both_themes_have_valid_nybble_palettes() {
        for theme in [&DARK_THEME, &LIGHT_THEME] {
            assert_eq!(theme.nybble_palette.len(), 16);
            for (i, c) in theme.nybble_palette.iter().enumerate() {
                assert!(
                    (0.0..=1.0).contains(&c.r),
                    "{:?} nybble_palette[{i}].r = {}",
                    if theme as *const _ == &DARK_THEME as *const _ {
                        "dark"
                    } else {
                        "light"
                    },
                    c.r,
                );
            }
        }
    }

    #[test]
    fn both_themes_have_valid_pattern_palettes() {
        for theme in [&DARK_THEME, &LIGHT_THEME] {
            assert_eq!(theme.pattern_bg_palette.len(), 16);
            assert_eq!(theme.pattern_fg_palette.len(), 16);
        }
    }

    #[test]
    fn scheme_lightness_not_nan() {
        assert!(!DARK_THEME.scheme_lightness.is_nan());
        assert!(!DARK_THEME.scheme_saturation.is_nan());
        assert!(!LIGHT_THEME.scheme_lightness.is_nan());
        assert!(!LIGHT_THEME.scheme_saturation.is_nan());
    }

    #[test]
    fn hsl_to_rgb_known_values() {
        // Black: any hue, s=0, l=0 → black
        let (r, g, b) = hsl_to_rgb(0.0, 0.0, 0.0);
        assert!((r - 0.0).abs() < 0.001);
        assert!((g - 0.0).abs() < 0.001);
        assert!((b - 0.0).abs() < 0.001);

        // White: any hue, s=0, l=1 → white
        let (r, g, b) = hsl_to_rgb(120.0, 0.0, 1.0);
        assert!((r - 1.0).abs() < 0.001);
        assert!((g - 1.0).abs() < 0.001);
        assert!((b - 1.0).abs() < 0.001);

        // Red: h=0°, s=1, l=0.5 → pure red
        let (r, g, b) = hsl_to_rgb(0.0, 1.0, 0.5);
        assert!((r - 1.0).abs() < 0.001);
        assert!((g - 0.0).abs() < 0.001);
        assert!((b - 0.0).abs() < 0.001);

        // Green: h=120°, s=1, l=0.5 → pure green
        let (r, g, b) = hsl_to_rgb(120.0, 1.0, 0.5);
        assert!((r - 0.0).abs() < 0.001);
        assert!((g - 1.0).abs() < 0.001);
        assert!((b - 0.0).abs() < 0.001);
    }

    #[test]
    fn theme_nybble_color_is_symmetric() {
        // Same high nybble → same colour (for non-0x00, non-0xFF).
        for b in 1..=0xFEu8 {
            let c_low = DARK_THEME.nybble_color(b);
            let high_nybble = (b >> 4) << 4;
            let other = high_nybble | 0x0F;
            if other > 0xFE {
                continue;
            }
            let c_high = DARK_THEME.nybble_color(other);
            assert!(
                (c_low.r - c_high.r).abs() < 0.001
                    && (c_low.g - c_high.g).abs() < 0.001
                    && (c_low.b - c_high.b).abs() < 0.001,
                "byte 0x{b:02X} and 0x{other:02X} should share nybble colour",
            );
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  Dark-theme WCAG AA normal-text (4.5:1) — matrix area
    // ═══════════════════════════════════════════════════════════════════════
    //
    // Every foreground that appears on `matrix_bg` in the hex/ascii columns
    // (see draw.rs cell-render loop for the exact layering).

    #[test]
    fn dark_theme_matrix_text_wcag_aa() {
        let t = &DARK_THEME;

        // Primary text colours on the bare matrix background.
        check("hex_fg", t.hex_fg, t.matrix_bg);
        check("ascii_fg", t.ascii_fg, t.matrix_bg);
        check("monochrome_fg", t.monochrome_fg, t.matrix_bg);
        check("annotation_fg", t.annotation_fg, t.matrix_bg);
        check("nybble_ff", t.nybble_ff, t.matrix_bg);
        check("default_null_dim", t.default_null_dim, t.matrix_bg);

        // 4 category colours.
        check("category_whitespace", t.category_whitespace, t.matrix_bg);
        check("category_printable", t.category_printable, t.matrix_bg);
        check("category_ctrl", t.category_ctrl, t.matrix_bg);
        check("category_non_ascii", t.category_non_ascii, t.matrix_bg);

        // Rainbow / heatmap scheme colours at three representative bytes.
        // scheme_lightness=0.70 is designed to pass WCAG AA on dark bg.
        for kind in ["rainbow", "heatmap"] {
            for byte in [0x00, 0x7F, 0xFF] {
                let col = match kind {
                    "rainbow" => t.rainbow_color(byte),
                    _ => t.heatmap_color(byte),
                };
                let label = format!("{kind}_0x{byte:02X}");
                check(&label, col, t.matrix_bg);
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  Dark-theme — dedicated nybble-palette contrast test
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn dark_theme_nybble_palette_wcag_aa() {
        let t = &DARK_THEME;
        // All 16 nybble-palette entries on matrix_bg.
        for (i, c) in t.nybble_palette.iter().enumerate() {
            check(&format!("nybble_palette[{i}]"), *c, t.matrix_bg);
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  Dark-theme WCAG AA (4.5:1) — overlays and gutters
    // ═══════════════════════════════════════════════════════════════════════
    //
    // Each overlay draws its own background then places text on top of it.

    #[test]
    fn dark_theme_overlays_wcag_aa() {
        let t = &DARK_THEME;

        // Address gutter.
        check("address_fg / address_gutter_bg", t.address_fg, t.address_gutter_bg);

        // Column header.
        check("header_fg / header_bg", t.header_fg, t.header_bg);

        // Selection / cursor.
        check("selection_fg / selection_bg", t.selection_fg, t.selection_bg);
        check("caret / cursor_bg", t.caret, t.cursor_bg);

        // Dirty, diff, editing overlays.
        check("dirty_fg / dirty_bg", t.dirty_fg, t.dirty_bg);
        check("diff_fg / diff_bg", t.diff_fg, t.diff_bg);
        check("edit_fg / edit_bg", t.edit_fg, t.edit_bg);

        // Search-match overlays (drawn on top of the cell bg).
        check("search_current_fg / search_current_bg", t.search_current_fg, t.search_current_bg);
        check("search_match_fg / search_match_bg", t.search_match_fg, t.search_match_bg);
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  Dark-theme WCAG AA (4.5:1) — modal & panel text
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn dark_theme_modal_text_wcag_aa() {
        let t = &DARK_THEME;
        check("modal_heading_fg / modal_bg", t.modal_heading_fg, t.modal_bg);
        check("modal_muted_fg / modal_bg", t.modal_muted_fg, t.modal_bg);
        check("modal_error_fg / modal_bg", t.modal_error_fg, t.modal_bg);
    }

    #[test]
    fn dark_theme_panel_text_wcag_aa() {
        let t = &DARK_THEME;
        check("pattern_count_fg / pattern_panel_bg", t.pattern_count_fg, t.pattern_panel_bg);
        check("export_info_fg / export_preview_bg", t.export_info_fg, t.export_preview_bg);
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  Dark-theme WCAG AA (4.5:1) — pattern overlay pairs
    // ═══════════════════════════════════════════════════════════════════════
    //
    // Each pattern index i pairs pattern_fg_palette[i] with
    // pattern_bg_palette[i] as a text-on-background pair.

    #[test]
    fn dark_theme_pattern_overlay_pairs_wcag_aa() {
        let t = &DARK_THEME;
        for i in 0..16 {
            let label = format!("pattern_pair[{i}]");
            check(&label, t.pattern_fg_palette[i], t.pattern_bg_palette[i]);
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  Dark-theme WCAG AA non-text (3:1) — UI elements
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn dark_theme_ui_elements_wcag_aa_non_text() {
        let t = &DARK_THEME;

        // Scrollbar.
        check_ui("scrollbar_thumb / scrollbar_bg", t.scrollbar_thumb, t.scrollbar_bg);
        check_ui(
            "scrollbar_thumb_hover / scrollbar_bg",
            t.scrollbar_thumb_hover,
            t.scrollbar_bg,
        );
        check_ui(
            "scrollbar_search_dot / scrollbar_bg",
            t.scrollbar_search_dot,
            t.scrollbar_bg,
        );
        check_ui(
            "scrollbar_cursor_dot / scrollbar_bg",
            t.scrollbar_cursor_dot,
            t.scrollbar_bg,
        );

        // Minimap.
        check_ui(
            "minimap_cursor_marker / minimap_bg",
            t.minimap_cursor_marker,
            t.minimap_bg,
        );
        check_ui(
            "minimap_dirty_pixel / minimap_bg",
            t.minimap_dirty_pixel,
            t.minimap_bg,
        );
        check_ui("minimap_diff_pixel / minimap_bg", t.minimap_diff_pixel, t.minimap_bg);

        // Borders / separators.
        check_ui("header_separator / header_bg", t.header_separator, t.header_bg);
        check_ui(
            "pattern_panel_border / pattern_panel_bg",
            t.pattern_panel_border,
            t.pattern_panel_bg,
        );
    }

    // ═══════════════════════════════════════════════════════════════════════
    //  Dark-theme luminance ordering (uses WCAG relative_luminance)
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn category_null_is_dimmer_than_printable() {
        let lum_null = relative_luminance(&DARK_THEME.category_color(0x00));
        let lum_print = relative_luminance(&DARK_THEME.category_color(0x41));
        assert!(
            lum_print > lum_null + 0.01,
            "NULL (rel_lum {lum_null:.4}) should be dimmer than printable \
             (rel_lum {lum_print:.4})",
        );
    }
}
