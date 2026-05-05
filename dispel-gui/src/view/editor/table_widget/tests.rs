use super::types::State;
use super::*;
use crate::view::editor::paragraph_cache::ParagraphCache;
use crate::view::editor::table_widget::style::cell_text_color;
use iced::{color, Size, Vector};

fn no_flags(_: usize) -> RowFlags {
    RowFlags::default()
}

fn col(width_px: f32) -> TableColumn {
    TableColumn {
        width_px,
        label: String::new(),
        sort: None,
        has_filter: false,
    }
}

#[test]
fn empty_table_does_not_panic() {
    let cache = ParagraphCache::default();
    let _w: TableWidget<'_, ()> = TableWidget::new(&[], &[], vec![], 42.0, no_flags, 24.0, cache);
}

#[test]
fn total_dimensions_include_id_column() {
    let cache = ParagraphCache::default();
    let display: Vec<Vec<String>> = vec![vec!["a".into(), "b".into()]; 5];
    let filtered: Vec<usize> = (0..5).collect();
    let cols = vec![col(100.0), col(200.0)];
    let w: TableWidget<'_, ()> =
        TableWidget::new(&display, &filtered, cols, 42.0, no_flags, 24.0, cache);
    assert_eq!(w.total_width(), 42.0 + 100.0 + 200.0);
    assert_eq!(w.total_height(), 5.0 * 24.0);
}

#[test]
fn cell_value_id_column_uses_orig_idx() {
    let display = vec![vec!["a".into()]; 3];
    let filtered = vec![2, 0, 1];
    let cols = vec![col(100.0)];
    let cache = ParagraphCache::default();
    let w: TableWidget<'_, ()> =
        TableWidget::new(&display, &filtered, cols, 42.0, no_flags, 24.0, cache);
    assert_eq!(w.cell_value(0, 0).as_deref(), Some("3"));
    assert_eq!(w.cell_value(1, 0).as_deref(), Some("1"));
    assert_eq!(w.cell_value(0, 1).as_deref(), Some("a"));
}

#[test]
fn sync_external_clamps_to_content() {
    let cache = ParagraphCache::default();
    let display: Vec<Vec<String>> = vec![vec!["a".into()]; 100];
    let filtered: Vec<usize> = (0..100).collect();
    let cols = vec![col(100.0)];
    let w: TableWidget<'_, ()> =
        TableWidget::new(&display, &filtered, cols, 42.0, no_flags, 24.0, cache)
            .external_offset(0.0, 100_000.0);
    let mut state = State::default();
    let bounds = Size::new(200.0, 240.0);
    w.sync_external(&mut state, bounds);
    assert_eq!(state.scroll_offset.y, 2184.0);
    assert_eq!(state.last_external, Some(Vector::new(0.0, 100_000.0)));
}

#[test]
fn sync_external_idempotent() {
    let cache = ParagraphCache::default();
    let display: Vec<Vec<String>> = vec![vec!["a".into()]; 50];
    let filtered: Vec<usize> = (0..50).collect();
    let cols = vec![col(100.0)];
    let w: TableWidget<'_, ()> =
        TableWidget::new(&display, &filtered, cols, 42.0, no_flags, 24.0, cache)
            .external_offset(10.0, 20.0);
    let mut state = State::default();
    let bounds = Size::new(200.0, 240.0);
    w.sync_external(&mut state, bounds);
    state.scroll_offset.y = 50.0;
    w.sync_external(&mut state, bounds);
    assert_eq!(state.scroll_offset.y, 50.0);
}

#[test]
fn cell_text_color_priority() {
    let f = RowFlags {
        current_highlight: true,
        highlighted: true,
        selected: true,
    };
    assert_eq!(cell_text_color(f), color!(0xffffff));
}
