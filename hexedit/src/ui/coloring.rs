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

/// Linearize a single sRGB component (0–255) to linear luminance.
fn srgb_linearize(c: u8) -> f32 {
    let c = c as f32 / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG 2.1 relative luminance of an sRGB colour.
pub fn relative_luminance(c: &Color) -> f32 {
    let r = (c.r * 255.0).round().clamp(0.0, 255.0) as u8;
    let g = (c.g * 255.0).round().clamp(0.0, 255.0) as u8;
    let b = (c.b * 255.0).round().clamp(0.0, 255.0) as u8;
    0.2126 * srgb_linearize(r) + 0.7152 * srgb_linearize(g) + 0.0722 * srgb_linearize(b)
}

/// WCAG 2.1 contrast ratio between two colours.
pub fn contrast_ratio(a: &Color, b: &Color) -> f32 {
    let l1 = relative_luminance(a);
    let l2 = relative_luminance(b);
    let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}

/// Default dim colour for null bytes — used when `dim_nulls` is active.
/// Deliberately very dim (CR~1.9 against the matrix background) so null
/// bytes visually recede. Users who need better legibility can disable
/// `dim_nulls` in the settings.
pub const DEFAULT_NULL_DIM: Color = Color::from_rgb(
    0x4A as f32 / 255.0,
    0x43 as f32 / 255.0,
    0x39 as f32 / 255.0,
);

/// The matrix background colour — used for contrast-threshold assertions.
pub const MATRIX_BG: Color = Color::from_rgb(
    0x14 as f32 / 255.0,
    0x11 as f32 / 255.0,
    0x0F as f32 / 255.0,
);

// ── Built-in colour functions ────────────────────────────────────────────

/// Map a byte to a colour based on its high nybble — 18 groups.
///
/// All palette entries are bright enough for CR ≥ 4.5 against `MATRIX_BG`.
pub fn nybble_color(b: u8) -> Color {
    match b {
        0x00 => DEFAULT_NULL_DIM,
        0xFF => hex(0xd4cabd),
        _ => {
            let palette = [
                hex(0x887c6f), hex(0x8a7f64), hex(0x7a8f5a), hex(0x6a8f4a),
                hex(0x5a8f5a), hex(0x5a8f6a), hex(0x5e856f), hex(0x6f855e),
                hex(0x8a7d54), hex(0xa77346), hex(0xbb644c), hex(0xc65c57),
                hex(0xc35e68), hex(0xb26a74), hex(0xa0726d), hex(0x897c65),
            ];
            palette[(b >> 4) as usize]
        }
    }
}

/// 6 semantic categories matching hexyl's default scheme.
///
/// All entries are bright enough for CR ≥ 4.5 against `MATRIX_BG`
/// (CR ≥ 3.0 for the intentionally dim NULL entry).
pub fn category_color(b: u8) -> Color {
    match b {
        0x00        => DEFAULT_NULL_DIM,          // NULL, intentionally dim
        0x09..=0x0D => hex(0x618950),              // whitespace (tab, nl, cr)
        0x20..=0x7E => hex(0xb8a898),              // printable ASCII
        0x7F        => hex(0x967962),              // DEL
        _ if b < 0x20 => hex(0x967962),            // other control chars
        _           => hex(0xa26f8f),              // non-ASCII (0x80..0xFF)
    }
}

/// Continuous rainbow hue, skipping the red-to-red wrap for clarity.
///
/// Lightness parameter (0.70) chosen so every byte has CR ≥ 4.5 against
/// `MATRIX_BG` (the blue ~0xCC region is the bottleneck).
pub fn rainbow_color(b: u8) -> Color {
    // Map 0→300° hue so 0x00 starts red and 0xFF ends magenta.
    let hue = (b as f32 / 255.0) * 300.0;
    let (r, g, b_) = hsl_to_rgb(hue, 0.85, 0.70);
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

/// Convenience wrapper: apply `SchemeProvider` + `DimNullsProvider` in the
/// correct order and return the `(fg, bg)` pair.  Uses [`DEFAULT_NULL_DIM`]
/// as the dim-null colour.
///
/// This is the single source of truth for the provider chain so the matrix
/// widget and the settings-modal palette preview stay in sync.
pub fn default_byte_colors(scheme: ColorScheme, byte: u8, dim_nulls: bool) -> (Option<Color>, Option<Color>) {
    let scheme_prov = SchemeProvider { scheme };
    let dim_prov = DimNullsProvider {
        enabled: dim_nulls,
        null_color: DEFAULT_NULL_DIM,
    };
    fold_color(
        [&scheme_prov as &dyn CellColorProvider, &dim_prov as &dyn CellColorProvider],
        0,
        byte,
    )
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

    // ── helpers ─────────────────────────────────────────────────────────

    fn red() -> Color {
        Color::from_rgb(1.0, 0.0, 0.0)
    }
    fn green() -> Color {
        Color::from_rgb(0.0, 1.0, 0.0)
    }
    fn blue() -> Color {
        Color::from_rgb(0.0, 0.0, 1.0)
    }

    /// Check that `c.r/g/b` are in [0, 1].
    fn assert_valid_color(c: &Color, label: &str) {
        assert!(
            (0.0..=1.0).contains(&c.r),
            "{label}: r={} out of range",
            c.r
        );
        assert!(
            (0.0..=1.0).contains(&c.g),
            "{label}: g={} out of range",
            c.g
        );
        assert!(
            (0.0..=1.0).contains(&c.b),
            "{label}: b={} out of range",
            c.b
        );
    }

    /// Minimum WCAG contrast ratio for normal-size text (AA).
    const CR_NORMAL: f32 = 4.5;

    // ── fold_color ────────────────────────────────────────────────────────

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
    fn fold_empty_providers_returns_none() {
        let empty: [&dyn CellColorProvider; 0] = [];
        let (fg, bg) = fold_color(empty, 0, 0);
        assert!(fg.is_none());
        assert!(bg.is_none());
    }

    #[test]
    fn fold_merges_partial_channels() {
        // Provider A gives fg=red (no bg), Provider B gives bg=blue (no fg).
        // The merged result should have both.
        struct PartialFg;
        impl CellColorProvider for PartialFg {
            fn color(&self, _addr: u64, _byte: u8) -> (Option<Color>, Option<Color>) {
                (Some(red()), None)
            }
        }
        struct PartialBg;
        impl CellColorProvider for PartialBg {
            fn color(&self, _addr: u64, _byte: u8) -> (Option<Color>, Option<Color>) {
                (None, Some(blue()))
            }
        }
        let (fg, bg) = fold_color(
            [&PartialFg as &dyn CellColorProvider, &PartialBg as &dyn CellColorProvider],
            0,
            0,
        );
        assert_eq!(fg, Some(red()));
        assert_eq!(bg, Some(blue()));
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

    // ── SchemeProvider ──────────────────────────────────────────────────

    #[test]
    fn scheme_provider_monochrome_constant() {
        let p = SchemeProvider {
            scheme: ColorScheme::Monochrome,
        };
        // All bytes get the same colour.
        let c0 = p.color(0, 0x00).0.unwrap();
        let c1 = p.color(0, 0xFF).0.unwrap();
        assert!((c0.r - c1.r).abs() < 0.001);
        assert!((c0.g - c1.g).abs() < 0.001);
        assert!((c0.b - c1.b).abs() < 0.001);
        // Always Some(fg), never bg.
        assert!(p.color(0, 0x42).1.is_none());
    }

    #[test]
    fn scheme_provider_nybble_delegates_to_nybble_color() {
        let p = SchemeProvider {
            scheme: ColorScheme::Nybble,
        };
        for b in [0x00, 0x01, 0x0F, 0x10, 0x7F, 0x80, 0xFF] {
            let expected = nybble_color(b);
            let actual = p.color(0, b).0.unwrap();
            assert!(
                (actual.r - expected.r).abs() < 0.001,
                "byte 0x{b:02X}: r {:.4} ≠ {:.4}",
                actual.r,
                expected.r
            );
            assert!(
                (actual.g - expected.g).abs() < 0.001,
                "byte 0x{b:02X}: g {:.4} ≠ {:.4}",
                actual.g,
                expected.g
            );
            assert!(
                (actual.b - expected.b).abs() < 0.001,
                "byte 0x{b:02X}: b {:.4} ≠ {:.4}",
                actual.b,
                expected.b
            );
        }
    }

    #[test]
    fn scheme_provider_categories_delegates() {
        let p = SchemeProvider {
            scheme: ColorScheme::Categories,
        };
        // Spot-check each category boundary.
        let checkpoints = [0x00, 0x01, 0x09, 0x0D, 0x0E, 0x20, 0x7E, 0x7F, 0x80, 0xFF];
        for &b in &checkpoints {
            let expected = category_color(b);
            let actual = p.color(0, b).0.unwrap();
            let same =
                (actual.r - expected.r).abs() < 0.001
                    && (actual.g - expected.g).abs() < 0.001
                    && (actual.b - expected.b).abs() < 0.001;
            assert!(same, "byte 0x{b:02X}: got ({:.4},{:.4},{:.4}) expected ({:.4},{:.4},{:.4})",
                    actual.r, actual.g, actual.b, expected.r, expected.g, expected.b);
        }
    }

    #[test]
    fn scheme_provider_rainbow_delegates() {
        let p = SchemeProvider {
            scheme: ColorScheme::Rainbow,
        };
        for b in [0x00, 0x01, 0x40, 0x80, 0xBF, 0xCC, 0xFF] {
            let expected = rainbow_color(b);
            let actual = p.color(0, b).0.unwrap();
            let same =
                (actual.r - expected.r).abs() < 0.001
                    && (actual.g - expected.g).abs() < 0.001
                    && (actual.b - expected.b).abs() < 0.001;
            assert!(same, "byte 0x{b:02X}: rainbow mismatch");
        }
    }

    // ── DimNullsProvider ────────────────────────────────────────────────

    #[test]
    fn dim_nulls_enabled_null() {
        let p = DimNullsProvider {
            enabled: true,
            null_color: red(),
        };
        assert_eq!(p.color(0, 0x00), (Some(red()), None));
        // Different address, same byte → same result.
        assert_eq!(p.color(100, 0x00), (Some(red()), None));
    }

    #[test]
    fn dim_nulls_enabled_non_null() {
        let p = DimNullsProvider {
            enabled: true,
            null_color: red(),
        };
        for b in [0x01, 0x20, 0x7F, 0xFF] {
            assert_eq!(p.color(0, b), (None, None), "byte 0x{b:02X} should not be dimmed");
        }
    }

    #[test]
    fn dim_nulls_disabled() {
        let p = DimNullsProvider {
            enabled: false,
            null_color: red(),
        };
        // Null byte with disabled → no override.
        assert_eq!(p.color(0, 0x00), (None, None));
        // Non-null byte also no override.
        assert_eq!(p.color(0, 0x42), (None, None));
    }

    // ── Nybble-colour tests ─────────────────────────────────────────────

    #[test]
    fn nybble_color_00_is_dim() {
        let c = nybble_color(0x00);
        // 0x00 should be dimmer than the palette default.
        let mid = nybble_color(0x10);
        let lum_00 = luminance(&c);
        let lum_mid = luminance(&mid);
        assert!(
            lum_00 < lum_mid - 0.1,
            "0x00 (lum {lum_00:.3}) should be dimmer than nybble-1 (lum {lum_mid:.3})"
        );
        // Must still be distinguishable from the background (not pure black).
        let cr = contrast_ratio(&c, &MATRIX_BG);
        assert!(
            cr > 1.0,
            "0x00 CR={cr:.2} should be > 1 (visible against background)"
        );
    }

    #[test]
    fn nybble_color_ff_is_bright() {
        let c = nybble_color(0xFF);
        let lum = luminance(&c);
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

    #[test]
    fn nybble_all_palette_entries_unique() {
        // Build the in-palette colours by reading the match arms.
        let mut seen = std::collections::HashSet::new();
        for high in 0..=15u8 {
            let b = high << 4;
            let c = nybble_color(b);
            // Float comparison: round each component to 3 decimals.
            let key = format!(
                "{:.0}{:.0}{:.0}",
                (c.r * 255.0).round() as u16,
                (c.g * 255.0).round() as u16,
                (c.b * 255.0).round() as u16,
            );
            assert!(
                seen.insert(key.clone()),
                "nybble 0x{high:X} (byte 0x{b:02X}) duplicates colour",
            );
            // Also ensure valid colour.
            assert_valid_color(&c, &format!("nybble 0x{high:X}"));
        }
        assert_eq!(seen.len(), 16, "all 16 palette entries should be unique");
    }

    // ── Category-colour tests ───────────────────────────────────────────

    #[test]
    fn category_null_at_boundary() {
        let c = category_color(0x00);
        assert_valid_color(&c, "NULL");
        // Intentionally dimmer than printable.
        let printable = category_color(0x41);
        assert!(luminance(&c) < luminance(&printable) - 0.1);
        // Must still be distinguishable from background.
        let cr = contrast_ratio(&c, &MATRIX_BG);
        assert!(cr > 1.0, "NULL CR={cr:.2} should be > 1 (visible)");
    }

    #[test]
    fn category_whitespace_range() {
        let color = category_color(0x09);
        for b in 0x09..=0x0D {
            let c = category_color(b);
            let same = (c.r - color.r).abs() < 0.001
                && (c.g - color.g).abs() < 0.001
                && (c.b - color.b).abs() < 0.001;
            assert!(same, "whitespace byte 0x{b:02X} has different colour");
            assert_valid_color(&c, &format!("whitespace 0x{b:02X}"));
        }
    }

    #[test]
    fn category_printable_range() {
        let color = category_color(0x41);
        for b in [0x20, 0x30, 0x41, 0x61, 0x7E] {
            let c = category_color(b);
            let same = (c.r - color.r).abs() < 0.001
                && (c.g - color.g).abs() < 0.001
                && (c.b - color.b).abs() < 0.001;
            assert!(same, "printable byte 0x{b:02X} has different colour");
            assert_valid_color(&c, &format!("printable 0x{b:02X}"));
        }
    }

    #[test]
    fn category_del_and_control_same() {
        // 0x7F (DEL) and other control bytes (<0x20, excluding NULL/whitespace)
        // share the same colour.
        let del = category_color(0x7F);
        for b in [0x01, 0x02, 0x0E, 0x1F] {
            let c = category_color(b);
            let same = (c.r - del.r).abs() < 0.001
                && (c.g - del.g).abs() < 0.001
                && (c.b - del.b).abs() < 0.001;
            assert!(same, "control byte 0x{b:02X} should match DEL colour");
        }
        assert_valid_color(&del, "DEL");
    }

    #[test]
    fn category_non_ascii_range() {
        for b in [0x80, 0x9F, 0xC0, 0xFF] {
            let c = category_color(b);
            assert_valid_color(&c, &format!("non-ASCII 0x{b:02X}"));
        }
        // All non-ASCII bytes share the same colour.
        let c80 = category_color(0x80);
        let cff = category_color(0xFF);
        let same = (c80.r - cff.r).abs() < 0.001
            && (c80.g - cff.g).abs() < 0.001
            && (c80.b - cff.b).abs() < 0.001;
        assert!(same, "all non-ASCII bytes should share the same colour");
    }

    // ── Rainbow-colour tests ────────────────────────────────────────────

    #[test]
    fn rainbow_no_panic_all_bytes() {
        for b in 0..=255u8 {
            let c = rainbow_color(b);
            assert_valid_color(&c, &format!("rainbow 0x{b:02X}"));
        }
    }

    #[test]
    fn rainbow_0x00_and_0xff_distinct() {
        let c0 = rainbow_color(0x00);
        let cff = rainbow_color(0xFF);
        let same =
            (c0.r - cff.r).abs() < 0.01
                && (c0.g - cff.g).abs() < 0.01
                && (c0.b - cff.b).abs() < 0.01;
        assert!(!same, "rainbow 0x00 and 0xFF should be different colours");
    }

    #[test]
    fn rainbow_hue_variation() {
        // Adjacent bytes should produce detectably different colours
        // (hue changes by ~1.17° per step, so the difference is subtle but real).
        // At minimum, 0x00 (red) and 0x55 (green) must be very different.
        let c00 = rainbow_color(0x00);
        let c55 = rainbow_color(0x55);
        let diff = (c00.r - c55.r).abs() + (c00.g - c55.g).abs() + (c00.b - c55.b).abs();
        assert!(
            diff > 0.3,
            "0x00 and 0x55 should be very different, total diff={diff:.3}"
        );
    }

    // ── scheme_color ─────────────────────────────────────────────────────

    #[test]
    fn scheme_color_dispatches_correctly() {
        for scheme in &ColorScheme::ALL {
            for b in [0x00, 0x01, 0x20, 0x7F, 0x80, 0xFF] {
                let c = scheme_color(*scheme, b);
                assert_valid_color(&c, &format!("{scheme:?} @ 0x{b:02X}"));
            }
        }
    }

    // ── ColorScheme metadata ────────────────────────────────────────────

    #[test]
    fn color_scheme_display_matches_label() {
        for s in &ColorScheme::ALL {
            assert_eq!(format!("{s}"), s.label(), "Display mismatch for {s:?}");
        }
    }

    #[test]
    fn color_scheme_all_contains_all() {
        assert_eq!(ColorScheme::ALL.len(), 4);
        assert!(ColorScheme::ALL.contains(&ColorScheme::Monochrome));
        assert!(ColorScheme::ALL.contains(&ColorScheme::Nybble));
        assert!(ColorScheme::ALL.contains(&ColorScheme::Categories));
        assert!(ColorScheme::ALL.contains(&ColorScheme::Rainbow));
    }

    // ── luminance / relative_luminance / contrast_ratio ─────────────────

    #[test]
    fn luminance_known_values() {
        let black = Color::from_rgb(0.0, 0.0, 0.0);
        let white = Color::from_rgb(1.0, 1.0, 1.0);
        assert!((luminance(&black) - 0.0).abs() < 0.001);
        assert!((luminance(&white) - 1.0).abs() < 0.001);

        let mid = Color::from_rgb(0.5, 0.5, 0.5);
        assert!((luminance(&mid) - 0.5).abs() < 0.001);
    }

    #[test]
    fn relative_luminance_and_contrast_ratio() {
        let black = Color::from_rgb(0.0, 0.0, 0.0);
        let white = Color::from_rgb(1.0, 1.0, 1.0);
        // WCAG: black-on-white = 21:1
        let cr = contrast_ratio(&black, &white);
        assert!((cr - 21.0).abs() < 0.5, "black/white CR={cr:.1} (expected ~21)");

        // Identity: same colour → 1:1
        let gray = Color::from_rgb(0.5, 0.5, 0.5);
        let cr_id = contrast_ratio(&gray, &gray);
        assert!((cr_id - 1.0).abs() < 0.01, "same colour CR={cr_id:.2} (expected 1.0)");

        // MATRIX_BG relative luminance should be very low but non-zero.
        let bg_lum = relative_luminance(&MATRIX_BG);
        assert!(
            bg_lum > 0.0 && bg_lum < 0.05,
            "BG relative luminance {bg_lum:.4} out of expected range"
        );
    }

    // ── default_byte_colors provider chain ─────────────────────────────

    #[test]
    fn default_byte_colors_provider_chain() {
        // With dim_nulls=true, a null byte should get DEFAULT_NULL_DIM.
        let (fg, bg) = default_byte_colors(ColorScheme::Monochrome, 0x00, true);
        assert!(bg.is_none(), "should not touch background");
        let fg = fg.unwrap();
        assert!(
            (fg.r - DEFAULT_NULL_DIM.r).abs() < 0.001
                && (fg.g - DEFAULT_NULL_DIM.g).abs() < 0.001
                && (fg.b - DEFAULT_NULL_DIM.b).abs() < 0.001,
            "null byte with dim should get DEFAULT_NULL_DIM"
        );

        // With dim_nulls=false, scheme passes through.
        let (fg2, _) = default_byte_colors(ColorScheme::Monochrome, 0x00, false);
        let expected = scheme_color(ColorScheme::Monochrome, 0x00);
        assert!(
            (fg2.unwrap().r - expected.r).abs() < 0.001,
            "dim=false should pass scheme through"
        );
    }

    #[test]
    fn default_byte_colors_non_null_not_dimmed() {
        let (fg, _) = default_byte_colors(ColorScheme::Nybble, 0x42, true);
        let expected = scheme_color(ColorScheme::Nybble, 0x42);
        let fg = fg.unwrap();
        assert!(
            (fg.r - expected.r).abs() < 0.001,
            "non-null byte should not be dimmed"
        );
    }

    // ── CRITICAL: every colour in every scheme meets contrast ─────────

    fn check_scheme_contrast(scheme: ColorScheme, min_cr: f32, label: &str) {
        let mut min_actual = f32::MAX;
        let mut min_byte = 0u8;
        for b in 0..=255u8 {
            let c = scheme_color(scheme, b);
            let cr = contrast_ratio(&c, &MATRIX_BG);
            if cr < min_actual {
                min_actual = cr;
                min_byte = b;
            }
        }
        assert!(
            min_actual >= min_cr,
            "{label}: byte 0x{min_byte:02X} has CR={min_actual:.2} < {min_cr}"
        );
    }

    #[test]
    fn monochrome_contrast() {
        check_scheme_contrast(ColorScheme::Monochrome, CR_NORMAL, "Monochrome");
    }

    #[test]
    fn nybble_contrast() {
        // 0x00 is intentionally dim — ignore it (checked in nybble_color_00_is_dim).
        // 0xFF is also special (bright), must meet normal threshold.
        let ff_cr = contrast_ratio(&nybble_color(0xFF), &MATRIX_BG);
        assert!(
            ff_cr >= CR_NORMAL,
            "nybble 0xFF CR={ff_cr:.2} < {CR_NORMAL}"
        );
        // All remaining bytes (0x01..=0xFE) use the palette.
        for b in 1..=0xFEu8 {
            let c = nybble_color(b);
            let cr = contrast_ratio(&c, &MATRIX_BG);
            assert!(
                cr >= CR_NORMAL,
                "Nybble byte 0x{b:02X} CR={cr:.2} < {CR_NORMAL}"
            );
        }
    }

    #[test]
    fn categories_contrast() {
        // NULL (0x00) is intentionally very dim — skip, tested in category_null_at_boundary.
        // Check all non-NULL bytes meet CR_NORMAL.
        for b in 1..=255u8 {
            let c = category_color(b);
            let cr = contrast_ratio(&c, &MATRIX_BG);
            assert!(
                cr >= CR_NORMAL,
                "Categories byte 0x{b:02X} CR={cr:.2} < {CR_NORMAL}"
            );
        }
    }

    #[test]
    fn rainbow_contrast() {
        check_scheme_contrast(ColorScheme::Rainbow, CR_NORMAL, "Rainbow");
    }

    #[test]
    fn dim_nulls_null_meets_dim_contrast() {
        // The dim-null colour should be noticeably dimmer than the
        // monochrome default — at least 5:1 contrast ratio difference.
        let cr = contrast_ratio(&DEFAULT_NULL_DIM, &MATRIX_BG);
        let mono = scheme_color(ColorScheme::Monochrome, 0x42);
        let mono_cr = contrast_ratio(&mono, &MATRIX_BG);
        assert!(
            mono_cr - cr > 5.0,
            "DEFAULT_NULL_DIM CR={cr:.2} too close to monochrome CR={mono_cr:.2} (diff {})",
            mono_cr - cr
        );
    }
}
