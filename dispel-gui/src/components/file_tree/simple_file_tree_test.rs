// Simple file tree test that can run independently
#![cfg(test)]

use super::file_tree::FileTree;
use super::tree_node::TreeNode;
use std::path::PathBuf;

#[test]
fn test_file_tree_basic_functionality() {
    // Test that we can create a file tree
    let tree = FileTree::default();
    assert!(tree.root.is_none());
    assert_eq!(tree.search_query, "");
}

#[test]
fn test_tree_node_creation() {
    let test_path = PathBuf::from("/test/file.txt");

    // Test creating a file node
    let file_node = TreeNode::File {
        path: test_path.clone(),
        name: "file.txt".to_string(),
        icon: "📄",
    };

    assert!(file_node.path().is_some());
    assert_eq!(file_node.path().unwrap(), &test_path);

    // Test creating a directory node
    let dir_path = PathBuf::from("/test/dir");
    let dir_node = TreeNode::Dir {
        path: dir_path.clone(),
        name: "dir".to_string(),
        expanded: false,
        children: Vec::new(),
    };

    assert!(dir_node.path().is_some());
    assert_eq!(dir_node.path().unwrap(), &dir_path);
}

#[test]
fn test_file_tree_context_menu() {
    use super::file_tree::ContextMenuState;

    let mut tree = FileTree::default();
    let test_path = PathBuf::from("/test/file.db");
    let test_position = iced::Point::new(100.0, 100.0);

    // Test setting context menu
    tree.context_menu = Some(ContextMenuState {
        position: test_position,
        file_path: test_path.clone(),
        is_visible: true,
    });

    assert!(tree.context_menu.is_some());
    let menu = tree.context_menu.as_ref().unwrap();
    assert!(menu.is_visible);
    assert_eq!(menu.file_path, test_path);
    assert_eq!(menu.position.x, 100.0);
    assert_eq!(menu.position.y, 100.0);
}
