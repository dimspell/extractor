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

use super::selection::Selection;

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
}
