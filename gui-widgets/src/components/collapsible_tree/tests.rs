//! Tests for the generic tree model and its filter/render behavior.

use super::*;
use iced::widget::text;
use iced_test::simulator;

/// Renders a node label with a concrete message type so the simulator can
/// infer the full widget type.
fn render_label<'a>(ctx: RenderContext<'a, &'static str>) -> Element<'a, ()> {
    text(*ctx.data).into()
}

fn sample_tree() -> TreeNode<&'static str> {
    TreeNode::branch(
        "root",
        vec![
            TreeNode::leaf("alpha"),
            TreeNode::branch("nested", vec![TreeNode::leaf("beta")]),
        ],
    )
}

#[test]
fn test_treenode_leaf_and_branch_shape() {
    let leaf = TreeNode::leaf("solo");
    assert!(!leaf.expanded);
    assert!(leaf.children.is_empty());
    assert_eq!(leaf.data, "solo");

    let branch = sample_tree();
    assert!(!branch.expanded);
    assert_eq!(branch.children.len(), 2);
    assert_eq!(branch.data, "root");
}

#[test]
fn test_treenode_toggle_returns_new_state() {
    let mut node = TreeNode::leaf("x");
    assert!(node.toggle());
    assert!(node.expanded);
    assert!(!node.toggle());
}

#[test]
fn test_treenode_find_mut_is_preorder() {
    let mut tree = sample_tree();
    let found = tree.find_mut(&|d| *d == "nested").expect("nested exists");
    assert_eq!(found.data, "nested");
    assert!(tree.find_mut(&|d| *d == "missing").is_none());
}

#[test]
fn test_treenode_for_each_visits_every_node() {
    let seen = std::cell::RefCell::new(Vec::new());
    sample_tree().for_each(&|node| seen.borrow_mut().push(node.data));
    assert_eq!(seen.into_inner(), vec!["root", "alpha", "nested", "beta"]);
}

#[test]
fn test_collapsible_tree_view_renders_expanded_nodes() {
    let mut tree = sample_tree();
    tree.expanded = true;
    tree.children[1].expanded = true;
    let nodes = [tree];

    let mut ui = simulator(CollapsibleTree::new(&nodes, render_label).view());

    for label in ["root", "alpha", "nested", "beta"] {
        ui.find(label).unwrap_or_else(|_| panic!("missing {label}"));
    }
}

#[test]
fn test_collapsible_tree_filter_keeps_ancestors_of_matches() {
    let mut tree = sample_tree();
    tree.expanded = true;
    tree.children[1].expanded = true;
    let nodes = [tree, TreeNode::leaf("unrelated")];

    let mut ui = simulator(
        CollapsibleTree::new(&nodes, render_label)
            .filter(|d| *d == "beta")
            .view(),
    );

    ui.find("root")
        .expect("ancestor of a match must stay visible");
    ui.find("nested")
        .expect("parent of a match must stay visible");
    ui.find("beta").expect("matching leaf must be visible");
    assert!(
        ui.find("alpha").is_err(),
        "non-matching sibling must be filtered out"
    );
    assert!(
        ui.find("unrelated").is_err(),
        "non-matching root must be filtered out"
    );
}

#[test]
fn test_collapsible_tree_no_match_renders_empty() {
    let nodes = [sample_tree()];
    let mut ui = simulator(
        CollapsibleTree::new(&nodes, render_label)
            .filter(|_| false)
            .view(),
    );
    assert!(ui.find("root").is_err());
}
