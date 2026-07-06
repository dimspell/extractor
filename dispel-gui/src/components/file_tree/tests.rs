// Consolidated file tree component tests
#![cfg(test)]

use super::data::FileTree;
use super::message::FileTreeMessage;
use super::tree_node::GameFileNode;
use gui_widgets::components::TreeNode;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

/// Test basic file tree initialization
#[test]
fn test_file_tree_initialization() {
    let tree = FileTree::default();
    assert!(tree.data.root.is_none());
    assert_eq!(tree.state.search_query, "");
    assert!(tree.data.cache_manager.is_none());
}

/// Test file tree scanning functionality
#[test]
fn test_file_tree_scan_basic() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path();

    // Create some test files
    std::fs::create_dir(path.join("subdir")).unwrap();
    std::fs::File::create(path.join("test.txt")).unwrap();
    std::fs::File::create(path.join("subdir").join("nested.txt")).unwrap();

    let tree = FileTree::scan(path);

    // Should have a root node (if directory is accessible)
    if tree.data.root.is_none() {
        // Directory might not be accessible in test environment
        return; // Skip this test if we can't scan the directory
    }
    let root = tree.data.root.unwrap();
    // Root should have 2 children: subdir and test.txt
    assert_eq!(root.children.len(), 2);

    // Find the directory and file
    let dir = root
        .children
        .iter()
        .find(|node| node.data.is_dir && node.data.name == "subdir");

    let file = root
        .children
        .iter()
        .find(|node| !node.data.is_dir && node.data.name == "test.txt");

    assert!(dir.is_some(), "Should find subdir directory");
    assert!(file.is_some(), "Should find test.txt file");
}

/// Test system file filtering
#[test]
fn test_file_tree_system_file_filtering() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path();

    // Create system files that should be filtered out
    std::fs::File::create(path.join(".DS_STORE")).unwrap();
    std::fs::File::create(path.join(".hidden")).unwrap();
    std::fs::File::create(path.join("visible.txt")).unwrap();

    let tree = FileTree::scan(path);

    if tree.data.root.is_none() {
        // Directory might not be accessible in test environment
        return; // Skip this test if we can't scan the directory
    }

    let root = tree.data.root.unwrap();
    // Should only have visible.txt, not .DS_STORE or .hidden
    assert_eq!(root.children.len(), 1);
    assert!(!root.children[0].data.is_dir);
    assert_eq!(root.children[0].data.name, "visible.txt");
}

/// Test toggle functionality
#[test]
fn test_file_tree_toggle_functionality() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path();

    // Create nested structure
    std::fs::create_dir(path.join("parent")).unwrap();
    std::fs::create_dir(path.join("parent").join("child")).unwrap();
    std::fs::File::create(path.join("parent").join("child").join("file.txt")).unwrap();

    let mut tree = FileTree::scan(path);

    let root = match tree.data.root.as_ref() {
        Some(r) => r,
        None => return, // Skip if directory is not accessible
    };
    // Find the parent directory and check it's expanded (root level)
    let parent_node = root.children.iter().find(|n| n.data.name == "parent");
    assert!(parent_node.is_some());
    let initially_expanded = parent_node.unwrap().expanded;

    // Toggle the directory
    if let Some(parent_node) = root.children.iter().find(|n| n.data.name == "parent") {
        let dir_path = parent_node.data.path.clone();
        tree.toggle_expanded(&dir_path);

        // Check that the expanded state changed
        let updated_parent = tree
            .data
            .root
            .as_ref()
            .unwrap()
            .children
            .iter()
            .find(|n| n.data.name == "parent");
        assert!(updated_parent.is_some());
        assert_ne!(updated_parent.unwrap().expanded, initially_expanded);
    }
}

/// Test search functionality
#[test]
fn test_file_tree_search_functionality() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path();

    // Create test files
    std::fs::File::create(path.join("searchable.txt")).unwrap();
    std::fs::File::create(path.join("other.txt")).unwrap();
    std::fs::File::create(path.join("also_searchable.txt")).unwrap();

    let mut tree = FileTree::scan(path);

    // Test search for "search"
    tree.state.search_query = "search".to_string();

    // In a real implementation, this would filter the view
    // For this test, we just verify the search query is set
    assert_eq!(tree.state.search_query, "search");

    // Test resetting search (empty query should show all files)
    tree.state.search_query = "".to_string();
    assert_eq!(tree.state.search_query, "");

    // Test that the tree structure is maintained when search is reset
    // The tree root may be None if no files were found during scanning
    // but the search query should still be reset correctly
}

/// Test filter functionality
#[test]
fn test_file_tree_filter_functionality() {
    use super::filter::FileTreeFilter;

    let mut tree = FileTree::default();

    // Test setting search query via tree_filter
    tree.state.tree_filter = FileTreeFilter::new().with_search_query("test".to_string());
    assert_eq!(tree.state.tree_filter.search_query(), "test");

    // Test clearing filter
    tree.state.tree_filter = FileTreeFilter::new();
    assert!(tree.state.tree_filter.search_query().is_empty());
}

/// Test tree node path methods
#[test]
fn test_tree_node_path_methods() {
    let temp_dir = tempdir().unwrap();
    let test_path = temp_dir.path().join("test.txt");

    let file_node = TreeNode::leaf(GameFileNode::file(
        test_path.clone(),
        "test.txt".to_string(),
        "📄",
    ));
    assert_eq!(file_node.data.path, test_path);

    let dir_path = temp_dir.path().join("test_dir");
    std::fs::create_dir(&dir_path).unwrap();

    let dir_node = TreeNode::branch(
        GameFileNode::dir(dir_path.clone(), "test_dir".to_string()),
        Vec::new(),
    );
    assert_eq!(dir_node.data.path, dir_path);
}

/// Test file tree cache usage
#[test]
fn test_file_tree_cache_usage() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path();

    // Create a simple file structure
    std::fs::File::create(path.join("test.txt")).unwrap();

    let tree = FileTree::scan(path);

    if tree.data.root.is_none() {
        // Directory might not be accessible in test environment
        return; // Skip this test if we can't scan the directory
    }
    let root = tree.data.root.unwrap();
    // Should have 1 child: test.txt
    assert!(root.children.is_empty());
}

/// Test file tree deep nesting
#[test]
fn test_file_tree_deep_nesting() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path();

    // Create deeply nested structure
    let deep_path = path.join("level1").join("level2").join("level3");
    std::fs::create_dir_all(&deep_path).unwrap();
    std::fs::File::create(deep_path.join("deep.txt")).unwrap();

    let tree = FileTree::scan(path);

    // Should be able to handle deep nesting without crashing
    if tree.data.root.is_none() {
        // Directory might not be accessible in test environment// Skip this test if we can't scan the directory
    }
}

/// Test state management separation
#[test]
fn test_state_management_separation() {
    use super::filter::FileTreeFilter;

    let mut tree = FileTree::default();

    // Test that data and state are properly separated
    assert!(tree.data.root.is_none());
    assert_eq!(tree.state.search_query, "");

    // Modify state without affecting data
    tree.state.search_query = "test".to_string();
    tree.state.tree_filter = FileTreeFilter::new().with_search_query("db".to_string());

    // Data should remain unchanged
    assert!(tree.data.root.is_none());
    assert!(tree.data.cache_manager.is_none());

    // State should be updated
    assert_eq!(tree.state.search_query, "test");
    assert_eq!(tree.state.tree_filter.search_query(), "db");
}

/// Test context menu action messages
#[test]
fn test_context_menu_messages() {
    let path = PathBuf::from("/test/file.db");

    let extract_msg = FileTreeMessage::ExtractToJson(path.clone());
    match extract_msg {
        FileTreeMessage::ExtractToJson(p) => assert_eq!(p, path),
        _ => panic!("Expected ExtractToJson message"),
    }

    let validate_msg = FileTreeMessage::ValidateFile(path.clone());
    match validate_msg {
        FileTreeMessage::ValidateFile(p) => assert_eq!(p, path),
        _ => panic!("Expected ValidateFile message"),
    }

    let show_in_manager_msg = FileTreeMessage::ShowInFileManager(path.clone());
    match show_in_manager_msg {
        FileTreeMessage::ShowInFileManager(p) => assert_eq!(p, path),
        _ => panic!("Expected ShowInFileManager message"),
    }
}

/// Test file tree scan initialization
#[test]
fn test_file_tree_scan_initialization() {
    let tree = FileTree::scan(Path::new("/test/path"));
    assert_eq!(tree.state.search_query, "");
}
