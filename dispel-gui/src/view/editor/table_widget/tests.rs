use super::types::State;
use super::*;
use crate::view::editor::table_widget::style::cell_text_color;
use gui_widgets::components::paragraph_cache::ParagraphCache;
use iced::{color, Rectangle, Size, Vector};

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

/// Build a `TableColumn` with a non‑empty label (used for accessibility
/// label‑composition tests).
fn named_col(width_px: f32, label: &str) -> TableColumn {
    TableColumn {
        width_px,
        label: label.to_string(),
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

// ── Accessibility-oriented tests ────────────────────────────────────

#[test]
fn col_positions_are_cumulative() {
    let cache = ParagraphCache::default();
    let display = vec![vec!["a".into(), "b".into()]; 1];
    let filtered = vec![0];
    let cols = vec![named_col(100.0, "A"), named_col(200.0, "B")];
    let w: TableWidget<'_, ()> =
        TableWidget::new(&display, &filtered, cols, 42.0, no_flags, 24.0, cache);

    let positions = w.col_positions();
    // n_cols = 1 (id) + 2 (data) = 3 → positions.len() = n_cols + 1 = 4
    assert_eq!(positions.len(), 4);
    assert!((positions[0] - 0.0).abs() < f32::EPSILON);
    assert!((positions[1] - 42.0).abs() < f32::EPSILON, "expect id col width");
    assert!((positions[2] - 142.0).abs() < f32::EPSILON, "expect id + 100");
    assert!((positions[3] - 342.0).abs() < f32::EPSILON, "expect id + 100 + 200");
}

#[test]
fn col_positions_empty_only_id_col() {
    let cache = ParagraphCache::default();
    let w: TableWidget<'_, ()> =
        TableWidget::new(&[] as &[Vec<String>], &[], vec![], 42.0, no_flags, 24.0, cache);

    let positions = w.col_positions();
    // Only the id column → 1 position
    assert_eq!(positions.len(), 2);
    assert!((positions[0] - 0.0).abs() < f32::EPSILON);
    assert!((positions[1] - 42.0).abs() < f32::EPSILON);
}

#[test]
fn body_bounds_starts_after_header() {
    let cache = ParagraphCache::default();
    let cols = vec![named_col(100.0, "X")];
    let w: TableWidget<'_, ()> =
        TableWidget::new(&[] as &[Vec<String>], &[], cols, 42.0, no_flags, 24.0, cache);

    let bounds = Rectangle {
        x: 10.0,
        y: 20.0,
        width: 400.0,
        height: 200.0,
    };
    let body = w.body_bounds(bounds);
    let hdr_h = w.header_height();

    assert!((body.x - bounds.x).abs() < f32::EPSILON);
    assert!((body.y - (bounds.y + hdr_h)).abs() < f32::EPSILON);
    assert!(body.height <= bounds.height - hdr_h);
}

#[test]
fn body_bounds_width_fits_without_scrollbar() {
    let cache = ParagraphCache::default();
    let cols = vec![named_col(100.0, "X")];
    let w: TableWidget<'_, ()> =
        TableWidget::new(&[] as &[Vec<String>], &[], cols, 42.0, no_flags, 24.0, cache);

    // Total width = 142, bounds width = 500 → content fits → no scrollbar
    let bounds = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 500.0,
        height: 100.0,
    };
    let body = w.body_bounds(bounds);
    assert!((body.width - 500.0).abs() < f32::EPSILON,
        "body width should equal bounds width when content fits");
}

#[test]
fn body_bounds_reserves_horizontal_scrollbar_in_height() {
    let cache = ParagraphCache::default();
    let cols = vec![named_col(600.0, "Wide")];
    let w: TableWidget<'_, ()> =
        TableWidget::new(&[] as &[Vec<String>], &[], cols, 42.0, no_flags, 24.0, cache);

    // Total width = 642 > 200 → horizontal scrollbar needed
    // No vertical scrollbar (height fits) → width is bounds.width
    // Horizontal scrollbar reduces available height
    let bounds = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
    };
    let body = w.body_bounds(bounds);
    // No vertical scrollbar → width unchanged
    assert!((body.width - 200.0).abs() < f32::EPSILON,
        "body width unchanged when vertical scrollbar not needed");
    // Horizontal scrollbar subtracted from height
    let expected_h = 100.0 - w.header_height() - super::SCROLLBAR_THICKNESS;
    assert!((body.height - expected_h).abs() < f32::EPSILON,
        "body height subtracts horizontal scrollbar thickness");
}

#[test]
fn all_rows_included_in_full_range() {
    let cache = ParagraphCache::default();
    let display: Vec<Vec<String>> = (0..50).map(|i| vec![format!("val{i}")]).collect();
    let filtered: Vec<usize> = (0..50).collect();
    let cols = vec![named_col(100.0, "Data")];
    let w: TableWidget<'_, ()> =
        TableWidget::new(&display, &filtered, cols, 42.0, no_flags, 24.0, cache);

    // accessibility() now iterates 0..n_rows — verify the full count
    assert_eq!(w.n_rows(), 50);
    assert_eq!(w.total_height(), 50.0 * 24.0);
}

#[test]
fn cell_label_format_components() {
    let cache = ParagraphCache::default();
    let display = vec![
        vec!["Short Sword".into(), "15".into(), "3.5".into()],
        vec!["Iron Shield".into(), "8".into(), "7.2".into()],
    ];
    let filtered: Vec<usize> = (0..2).collect();
    let cols = vec![
        named_col(80.0, "Name"),
        named_col(60.0, "Damage"),
        named_col(60.0, "Weight"),
    ];
    let w: TableWidget<'_, ()> =
        TableWidget::new(&display, &filtered, cols, 42.0, no_flags, 24.0, cache);

    // Verify the data that accessibility() composes into labels:
    //   col 0 (id)      → "#: {orig_idx+1}"
    //   col 1 (Name)    → "Name: {value}"
    //   col 2 (Damage)  → "Damage: {value}"
    //   col 3 (Weight)  → "Weight: {value}"

    // Row 0
    assert_eq!(w.cell_value(0, 0).as_deref(), Some("1"), "id col → #: 1");
    assert_eq!(w.cell_value(0, 1).as_deref(), Some("Short Sword"));
    assert_eq!(w.cell_value(0, 2).as_deref(), Some("15"));
    assert_eq!(w.cell_value(0, 3).as_deref(), Some("3.5"));

    // Row 1
    assert_eq!(w.cell_value(1, 0).as_deref(), Some("2"), "id col → #: 2");
    assert_eq!(w.cell_value(1, 1).as_deref(), Some("Iron Shield"));
    assert_eq!(w.cell_value(1, 2).as_deref(), Some("8"));
    assert_eq!(w.cell_value(1, 3).as_deref(), Some("7.2"));

}

#[test]
fn scroll_target_y_clamps_to_content_height() {
    let cache = ParagraphCache::default();
    let display: Vec<Vec<String>> = (0..30).map(|i| vec![format!("v{i}")]).collect();
    let filtered: Vec<usize> = (0..30).collect();
    let cols = vec![named_col(100.0, "X")];
    let w: TableWidget<'_, ()> =
        TableWidget::new(&display, &filtered, cols, 42.0, no_flags, 24.0, cache);

    // Simulate the clamping that accessibility_action does:
    let row = 29usize;
    let target_y = row as f32 * w.row_height;         // 29 × 24 = 696
    let body_h = 240.0_f32;
    let max_scroll = (w.total_height() - body_h).max(0.0);
    let clamped = target_y.clamp(0.0, max_scroll);

    // total_height = 30 × 24 = 720, body_h = 240 → max_scroll = 480
    assert!((max_scroll - 480.0).abs() < f32::EPSILON,
        "max_scroll = total_height - body_height");
    assert!((clamped - 480.0).abs() < f32::EPSILON,
        "clamped to max_scroll when target exceeds it");
}
