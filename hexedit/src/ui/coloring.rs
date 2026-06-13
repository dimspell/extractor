//! Pluggable cell-coloring system for the hex matrix.
//!
//! v1 ships the trait + a small set of providers but does not yet replace
//! the matrix's hard-coded selection/cursor/dirty colors. The intent is for
//! follow-up commits — structure overlays, search hits, hover row/col — to
//! plug into [`fold_color`] without further widget surgery.
//!
//! Providers return `(Option<fg>, Option<bg>)`: each layer can opt out of
//! one or both decisions, and later providers in the chain win.

use std::collections::BTreeSet;
use std::ops::RangeInclusive;

use iced::Color;

use crate::selection::Selection;

/// One layer of the coloring chain. Each provider sees `(addr, byte)` for a
/// single cell and may contribute a foreground and/or background color.
pub trait CellColorProvider {
    fn color(&self, addr: u64, byte: u8) -> (Option<Color>, Option<Color>);
}

/// Fold a chain of providers into a final `(fg, bg)` pair, with each layer
/// overriding the previous when it returns `Some`.
pub fn fold_color<'a>(
    providers: impl IntoIterator<Item = &'a dyn CellColorProvider>,
    addr: u64,
    byte: u8,
) -> (Option<Color>, Option<Color>) {
    let mut fg = None;
    let mut bg = None;
    for p in providers {
        let (f, b) = p.color(addr, byte);
        if f.is_some() {
            fg = f;
        }
        if b.is_some() {
            bg = b;
        }
    }
    (fg, bg)
}

// ── Builtin providers ─────────────────────────────────────────────────────

/// Highlights every byte the user has overwritten since load.
pub struct DirtyProvider<'a> {
    pub dirty: &'a BTreeSet<u64>,
    pub fg: Color,
    pub bg: Color,
}

impl CellColorProvider for DirtyProvider<'_> {
    fn color(&self, addr: u64, _byte: u8) -> (Option<Color>, Option<Color>) {
        if self.dirty.contains(&addr) {
            (Some(self.fg), Some(self.bg))
        } else {
            (None, None)
        }
    }
}

/// Highlights bytes that differ from a vanilla snapshot.
pub struct DiffVsVanillaProvider<'a> {
    pub diff: &'a BTreeSet<u64>,
    pub fg: Color,
    pub bg: Color,
}

impl CellColorProvider for DiffVsVanillaProvider<'_> {
    fn color(&self, addr: u64, _byte: u8) -> (Option<Color>, Option<Color>) {
        if self.diff.contains(&addr) {
            (Some(self.fg), Some(self.bg))
        } else {
            (None, None)
        }
    }
}

/// Selection range and (separately) cursor cell.
pub struct SelectionProvider {
    pub range: RangeInclusive<u64>,
    pub cursor: u64,
    pub fg: Color,
    pub bg: Color,
    pub cursor_bg: Color,
}

impl SelectionProvider {
    pub fn from_selection(sel: Selection, fg: Color, bg: Color, cursor_bg: Color) -> Self {
        Self {
            range: sel.range(),
            cursor: sel.cursor,
            fg,
            bg,
            cursor_bg,
        }
    }
}

impl CellColorProvider for SelectionProvider {
    fn color(&self, addr: u64, _byte: u8) -> (Option<Color>, Option<Color>) {
        if !self.range.contains(&addr) {
            return (None, None);
        }
        let bg = if addr == self.cursor {
            self.cursor_bg
        } else {
            self.bg
        };
        (Some(self.fg), Some(bg))
    }
}

// ── Color scheme — which rule maps byte → fg colour ───────────────────────

/// Which colour scheme the hex matrix uses for its default byte foreground.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorScheme {
    /// No extra colouring — the original monochrome look.
    Monochrome,
    /// 18 groups: one per high nybble plus special `00` / `FF`.
    Nybble,
    /// 6 semantic categories (NULL / printable / whitespace / control / non-ASCII).
    Categories,
    /// Continuous rainbow hue across the full 0x00…0xFF range.
    Rainbow,
}

impl ColorScheme {
    pub const ALL: [ColorScheme; 4] = [
        ColorScheme::Monochrome,
        ColorScheme::Nybble,
        ColorScheme::Categories,
        ColorScheme::Rainbow,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            ColorScheme::Monochrome => "Monochrome",
            ColorScheme::Nybble => "Nybble (18 groups)",
            ColorScheme::Categories => "Categories (6 groups)",
            ColorScheme::Rainbow => "Rainbow gradient",
        }
    }
}

impl std::fmt::Display for ColorScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

// ── Color helpers ─────────────────────────────────────────────────────────

fn hex(c: u32) -> Color {
    let r = ((c >> 16) & 0xFF) as f32 / 255.0;
    let g = ((c >> 8) & 0xFF) as f32 / 255.0;
    let b = (c & 0xFF) as f32 / 255.0;
    Color::from_rgb(r, g, b)
}

/// Perceived luminance (rec. 709 weights) — used for colour-brightness tests.
pub fn luminance(c: &Color) -> f32 {
    0.299 * c.r + 0.587 * c.g + 0.114 * c.b
}

// ── Built-in colour functions ────────────────────────────────────────────

/// Map a byte to a colour based on its high nybble — 18 groups.
pub fn nybble_color(b: u8) -> Color {
    match b {
        0x00 => hex(0x4a4339),
        0xFF => hex(0xd4cabd),
        _ => {
            let palette = [
                hex(0x7a6f64), hex(0x8a7f64), hex(0x7a8f5a), hex(0x6a8f4a),
                hex(0x5a8f5a), hex(0x5a8f6a), hex(0x5a7f6a), hex(0x6a7f5a),
                hex(0x7a6f4a), hex(0x8a5f3a), hex(0x9a4f3a), hex(0xaa3f3a),
                hex(0xaa3f4a), hex(0x9a4f5a), hex(0x8a5f5a), hex(0x7a6f5a),
            ];
            palette[(b >> 4) as usize]
        }
    }
}

/// 6 semantic categories matching hexyl's default scheme.
pub fn category_color(b: u8) -> Color {
    match b {
        0x00        => hex(0x4a4339), // NULL
        0x09..=0x0D => hex(0x5a7f4a), // whitespace (tab, newline, cr, etc.)
        0x20..=0x7E => hex(0xb8a898), // printable ASCII
        0x7F        => hex(0x8a6f5a), // DEL
        _ if b < 0x20 => hex(0x8a6f5a), // other control chars
        _           => hex(0x7a4f6a), // non-ASCII (0x80..0xFF)
    }
}

/// Continuous rainbow hue, skipping the red-to-red wrap for clarity.
pub fn rainbow_color(b: u8) -> Color {
    // Map 0→300° hue so 0x00 starts blue-violet and 0xFF ends magenta.
    let hue = (b as f32 / 255.0) * 300.0;
    let (r, g, b_) = hsl_to_rgb(hue, 0.85, 0.55);
    Color::from_rgb(r, g, b_)
}

/// Convert HSL to RGB, all components in 0..1 range.
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

/// Pick the colour for a byte under the given scheme.
pub fn scheme_color(scheme: ColorScheme, b: u8) -> Color {
    match scheme {
        ColorScheme::Monochrome => hex(0xd4cabd),
        ColorScheme::Nybble => nybble_color(b),
        ColorScheme::Categories => category_color(b),
        ColorScheme::Rainbow => rainbow_color(b),
    }
}

// ── Providers for the CellColorProvider chain ─────────────────────────────

/// Applies a [`ColorScheme`] as a foreground provider in the chain.
/// Always returns `Some(fg)` and never touches the background.
pub struct SchemeProvider {
    pub scheme: ColorScheme,
}

impl CellColorProvider for SchemeProvider {
    fn color(&self, _addr: u64, byte: u8) -> (Option<Color>, Option<Color>) {
        (Some(scheme_color(self.scheme, byte)), None)
    }
}

/// Optionally dims `0x00` bytes by overriding their foreground.
/// Designed to sit *after* a scheme provider in the chain so it wins for
/// null bytes without affecting other values.
pub struct DimNullsProvider {
    pub enabled: bool,
    pub null_color: Color,
}

impl CellColorProvider for DimNullsProvider {
    fn color(&self, _addr: u64, byte: u8) -> (Option<Color>, Option<Color>) {
        if self.enabled && byte == 0x00 {
            (Some(self.null_color), None)
        } else {
            (None, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use iced::Color;

    fn red() -> Color {
        Color::from_rgb(1.0, 0.0, 0.0)
    }
    fn green() -> Color {
        Color::from_rgb(0.0, 1.0, 0.0)
    }
    fn blue() -> Color {
        Color::from_rgb(0.0, 0.0, 1.0)
    }

    #[test]
    fn fold_returns_none_when_no_provider_contributes() {
        let dirty = BTreeSet::new();
        let p = DirtyProvider {
            dirty: &dirty,
            fg: red(),
            bg: blue(),
        };
        let (fg, bg) = fold_color([&p as &dyn CellColorProvider], 0, 0);
        assert!(fg.is_none());
        assert!(bg.is_none());
    }

    #[test]
    fn dirty_provider_paints_only_dirty_addresses() {
        let mut dirty = BTreeSet::new();
        dirty.insert(7);
        let p = DirtyProvider {
            dirty: &dirty,
            fg: red(),
            bg: blue(),
        };
        assert_eq!(p.color(7, 0), (Some(red()), Some(blue())));
        assert_eq!(p.color(8, 0), (None, None));
    }

    #[test]
    fn later_layer_overrides_earlier() {
        let mut dirty = BTreeSet::new();
        dirty.insert(5);
        let mut diff = BTreeSet::new();
        diff.insert(5);
        let p1 = DirtyProvider {
            dirty: &dirty,
            fg: red(),
            bg: red(),
        };
        let p2 = DiffVsVanillaProvider {
            diff: &diff,
            fg: green(),
            bg: blue(),
        };
        let (fg, bg) = fold_color(
            [&p1 as &dyn CellColorProvider, &p2 as &dyn CellColorProvider],
            5,
            0,
        );
        assert_eq!(fg, Some(green()));
        assert_eq!(bg, Some(blue()));
    }

    #[test]
    fn selection_provider_distinguishes_cursor_from_range() {
        let sel = Selection {
            anchor: 10,
            cursor: 12,
        };
        let p = SelectionProvider::from_selection(sel, red(), blue(), green());
        assert_eq!(p.color(10, 0).1, Some(blue()));
        assert_eq!(p.color(11, 0).1, Some(blue()));
        assert_eq!(p.color(12, 0).1, Some(green()));
        assert_eq!(p.color(13, 0), (None, None));
    }

    // ── Nybble-colour tests ─────────────────────────────────────────────

    #[test]
    fn nybble_color_00_is_dim() {
        let c = nybble_color(0x00);
        // Should be noticeably darker (lower luminance) than the default hex
        // colour to make null bytes visually fade.
        let lum = 0.299 * c.r + 0.587 * c.g + 0.114 * c.b;
        assert!(lum < 0.3, "0x00 should be dim, got luminance {lum}");
    }

    #[test]
    fn nybble_color_ff_is_bright() {
        let c = nybble_color(0xFF);
        let lum = 0.299 * c.r + 0.587 * c.g + 0.114 * c.b;
        assert!(lum > 0.6, "0xFF should be bright, got luminance {lum}");
    }

    #[test]
    fn nybble_color_same_high_nybble_equal() {
        let c10 = nybble_color(0x10);
        let c1f = nybble_color(0x1F);
        assert!(
            (c10.r - c1f.r).abs() < 0.001
                && (c10.g - c1f.g).abs() < 0.001
                && (c10.b - c1f.b).abs() < 0.001,
            "0x10 and 0x1F should have the same colour"
        );
    }

    #[test]
    fn nybble_color_adjacent_nybbles_different() {
        let c0 = nybble_color(0x0F); // nybble 0
        let c1 = nybble_color(0x10); // nybble 1
        let same = (c0.r - c1.r).abs() < 0.001
            && (c0.g - c1.g).abs() < 0.001
            && (c0.b - c1.b).abs() < 0.001;
        assert!(!same, "different nybbles should have different colours");
    }

    #[test]
    fn nybble_color_00_and_0x_are_distinct() {
        let c00 = nybble_color(0x00);
        let c0x = nybble_color(0x0F);
        let same = (c00.r - c0x.r).abs() < 0.001
            && (c00.g - c0x.g).abs() < 0.001
            && (c00.b - c0x.b).abs() < 0.001;
        assert!(!same, "0x00 and 0x0F should have different colours");
    }
}
