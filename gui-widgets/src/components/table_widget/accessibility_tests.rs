//! End-to-end accesskit tree/action tests for the table widget.
//!
//! These drive the real `Simulator::accessibility_tree` /
//! `Simulator::accessibility_action` path instead of re-implementing the
//! node-id → (row, column) protocol in the test itself.

#![cfg(feature = "accessibility")]

use super::*;
use crate::components::paragraph_cache::ParagraphCache;
use iced::advanced::graphics::core::accessibility::accesskit;
use iced_test::simulator;

fn build_widget<'a>(display: &'a [Vec<String>], filtered: &'a [usize]) -> TableWidget<'a, String> {
    let columns = vec![TableColumn {
        width_px: 100.0,
        label: "Name".into(),
        sort: None,
        has_filter: false,
    }];
    TableWidget::new(
        display,
        filtered,
        columns,
        42.0,
        |_| RowFlags::default(),
        24.0,
        ParagraphCache::default(),
    )
    .accessible_label("Items")
    .on_select(|row| format!("select-{row}"))
    .on_scroll(|x, y, _h| format!("scroll-{x}-{y}"))
}

fn sample_data() -> (Vec<Vec<String>>, Vec<usize>) {
    (
        vec![vec!["alpha".to_string()], vec!["beta".to_string()]],
        vec![0, 1],
    )
}

#[test]
fn test_accessibility_tree_builds_table_hierarchy() {
    let (display, filtered) = sample_data();
    let element: iced::Element<'_, String> = build_widget(&display, &filtered).into();
    let mut ui = simulator(element);
    let tree = ui.accessibility_tree();

    let count = |role| {
        tree.nodes
            .iter()
            .filter(|(_, node)| node.role() == role)
            .count()
    };
    assert_eq!(count(accesskit::Role::Table), 1, "one table root");
    assert_eq!(
        count(accesskit::Role::ColumnHeader),
        2,
        "id + one data column"
    );
    assert_eq!(count(accesskit::Role::Row), 3, "header + two data rows");
    assert_eq!(count(accesskit::Role::Cell), 4, "2 rows × 2 columns");

    let table = tree
        .nodes
        .iter()
        .find(|(_, node)| node.role() == accesskit::Role::Table)
        .map(|(id, node)| (*id, node))
        .expect("table node");
    assert_eq!(table.1.label(), Some("Items"));
    assert_eq!(table.1.row_count(), Some(3));
    assert_eq!(table.1.column_count(), Some(2));
}

#[test]
fn test_accessibility_action_maps_cell_node_to_row_callback() {
    let (display, filtered) = sample_data();
    let element: iced::Element<'_, String> = build_widget(&display, &filtered).into();
    let mut ui = simulator(element);
    let tree = ui.accessibility_tree();

    // The data cell for row index 1 (original index 1) carries the composed
    // label "Name: beta". The two-phase protocol must map its NodeId back to
    // visible row 1, not to the column or the grid.
    let target = tree
        .nodes
        .iter()
        .find(|(_, node)| {
            node.role() == accesskit::Role::Cell && node.label() == Some("Name: beta")
        })
        .map(|(id, _)| *id)
        .expect("beta cell node");

    ui.accessibility_action(&accesskit::ActionRequest {
        action: accesskit::Action::ScrollIntoView,
        target_tree: accesskit::TreeId::ROOT,
        target_node: target,
        data: None,
    });

    let messages: Vec<String> = ui.into_messages().collect();
    assert_eq!(
        messages,
        vec!["select-1".to_string()],
        "Focus/ScrollIntoView on a cell must emit on_select for its row"
    );
}

#[test]
fn test_accessibility_action_ignores_unmapped_node_id() {
    let (display, filtered) = sample_data();
    let element: iced::Element<'_, String> = build_widget(&display, &filtered).into();
    let mut ui = simulator(element);

    // A NodeId that was never registered in cell_node_map must not select a row.
    ui.accessibility_action(&accesskit::ActionRequest {
        action: accesskit::Action::ScrollIntoView,
        target_tree: accesskit::TreeId::ROOT,
        target_node: accesskit::NodeId(u64::MAX),
        data: None,
    });

    let messages: Vec<String> = ui.into_messages().collect();
    assert!(messages.is_empty(), "unknown node id must be a no-op");
}
