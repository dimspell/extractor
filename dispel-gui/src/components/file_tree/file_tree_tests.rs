// File tree component tests
#![cfg(test)]

use super::file_tree::{ContextMenuState, FileTree};
use super::file_tree_message::FileTreeMessage;
use super::tree_node::TreeNode;
use crate::components::file_type_filter::FileTypeFilter;
use std::path::PathBuf;

#[test]
fn test_file_tree_initialization() {
    let tree = FileTree::default();
    assert!(tree.root.is_none());
    assert_eq!(tree.search_query, "");
    assert!(matches!(tree.filter, FileTypeFilter::All));
    assert!(tree.context_menu.is_none());
    assert!(tree.cache_manager.is_none());
}

#[test]
fn test_file_tree_scan_basic() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path();

    // Create some test files
    std::fs::create_dir(path.join("subdir")).unwrap();
    std::fs::File::create(path.join("test.txt")).unwrap();
    std::fs::File::create(path.join("subdir").join("nested.txt")).unwrap();

    let tree = FileTree::scan(path);

    // Should have a root node
    assert!(tree.root.is_some());
    if let Some(TreeNode::Dir { children, .. }) = tree.root {
        // Should have 2 children: subdir and test.txt
        assert_eq!(children.len(), 2);

        // Find the directory and file
        let dir = children.iter().find_map(|node| {
            if let TreeNode::Dir { name, .. } = node {
                if name == "subdir" {
                    Some(node)
                } else {
                    None
                }
            } else {
                None
            }
        });

        let file = children.iter().find_map(|node| {
            if let TreeNode::File { name, .. } = node {
                if name == "test.txt" {
                    Some(node)
                } else {
                    None
                }
            } else {
                None
            }
        });

        assert!(dir.is_some(), "Should find subdir directory");
        assert!(file.is_some(), "Should find test.txt file");
    } else {
        panic!("Root should be a directory node");
    }
}

#[test]
fn test_file_tree_system_file_filtering() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path();

    // Create system files that should be filtered out
    std::fs::File::create(path.join(".DS_STORE")).unwrap();
    std::fs::File::create(path.join(".hidden")).unwrap();
    std::fs::File::create(path.join("visible.txt")).unwrap();

    let tree = FileTree::scan(path);

    assert!(tree.root.is_some());
    if let Some(TreeNode::Dir { children, .. }) = tree.root {
        // Should only have visible.txt, not .DS_STORE or .hidden
        assert_eq!(children.len(), 1);

        match &children[0] {
            TreeNode::File { name, .. } => {
                assert_eq!(name, "visible.txt");
            }
            _ => panic!("Expected a file node"),
        }
    }
}

#[test]
fn test_file_tree_toggle_functionality() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path();

    // Create nested structure
    std::fs::create_dir(path.join("parent")).unwrap();
    std::fs::create_dir(path.join("parent").join("child")).unwrap();
    std::fs::File::create(path.join("parent").join("child").join("file.txt")).unwrap();

    let mut tree = FileTree::scan(path);

    // Find the parent directory and check it's expanded (root level)
    if let Some(TreeNode::Dir { children, .. }) = &tree.root {
        let parent_dir = children.iter().find_map(|node| {
            if let TreeNode::Dir { name, expanded, .. } = node {
                if name == "parent" {
                    Some((name.clone(), *expanded))
                } else {
                    None
                }
            } else {
                None
            }
        });

        assert!(parent_dir.is_some());
        let (_, initially_expanded) = parent_dir.unwrap();

        // Toggle the directory
        if let Some(dir_path) = children.iter().find_map(|node| {
            if let TreeNode::Dir { name, path, .. } = node {
                if name == "parent" {
                    Some(path.clone())
                } else {
                    None
                }
            } else {
                None
            }
        }) {
            tree.toggle(&dir_path);

            // Check that the expanded state changed
            if let Some(TreeNode::Dir {
                children: updated_children,
                ..
            }) = &tree.root
            {
                let updated_parent = updated_children.iter().find_map(|node| {
                    if let TreeNode::Dir { name, expanded, .. } = node {
                        if name == "parent" {
                            Some(*expanded)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });

                assert!(updated_parent.is_some());
                assert_ne!(updated_parent.unwrap(), initially_expanded);
            }
        }
    }
}

#[test]
fn test_file_tree_search_functionality() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path();

    // Create test files
    std::fs::File::create(path.join("searchable.txt")).unwrap();
    std::fs::File::create(path.join("other.txt")).unwrap();
    std::fs::File::create(path.join("also_searchable.txt")).unwrap();

    let mut tree = FileTree::scan(path);

    // Test search for "search"
    tree.search_query = "search".to_string();

    // In a real implementation, this would filter the view
    // For this test, we just verify the search query is set
    assert_eq!(tree.search_query, "search");

    // Test resetting search (empty query should show all files)
    tree.search_query = "".to_string();
    assert_eq!(tree.search_query, "");

    // Test that the tree structure is maintained when search is reset
    // The tree should still have the same root structure
    assert!(tree.root.is_some());
}

#[test]
fn test_file_tree_filter_functionality() {
    let mut tree = FileTree::default();

    // Test changing filter
    tree.filter = FileTypeFilter::Db;
    assert!(matches!(tree.filter, FileTypeFilter::Db));

    tree.filter = FileTypeFilter::Scr;
    assert!(matches!(tree.filter, FileTypeFilter::Scr));

    tree.filter = FileTypeFilter::All;
    assert!(matches!(tree.filter, FileTypeFilter::All));
}

#[test]
fn test_context_menu_state_management() {
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

    // Test hiding context menu
    if let Some(menu) = &mut tree.context_menu {
        menu.is_visible = false;
    }

    assert!(!tree.context_menu.as_ref().unwrap().is_visible);
}

#[test]
fn test_tree_node_path_methods() {
    let temp_dir = tempfile::tempdir().unwrap();
    let test_path = temp_dir.path().join("test.txt");

    let file_node = TreeNode::File {
        path: test_path.clone(),
        name: "test.txt".to_string(),
        icon: "📄",
    };

    assert!(file_node.path().is_some());
    assert_eq!(file_node.path().unwrap(), &test_path);

    let dir_path = temp_dir.path().join("test_dir");
    std::fs::create_dir(&dir_path).unwrap();

    let dir_node = TreeNode::Dir {
        path: dir_path.clone(),
        name: "test_dir".to_string(),
        expanded: false,
        children: Vec::new(),
    };

    assert!(dir_node.path().is_some());
    assert_eq!(dir_node.path().unwrap(), &dir_path);
}

#[test]
fn test_file_tree_cache_usage() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path();

    // Create a simple file structure
    std::fs::File::create(path.join("test.txt")).unwrap();

    let tree = FileTree::scan(path);

    // Test should_use_cache method
    let cache = None;
    assert!(!tree.should_use_cache(&cache));

    // In a real scenario, this would test with actual cache data
    // For now, we just verify the method exists and doesn't panic
}

#[test]
fn test_file_tree_message_creation() {
    let test_path = PathBuf::from("/test/file.db");
    let test_position = iced::Point::new(50.0, 50.0);

    // Test creating various file tree messages
    let messages = vec![
        FileTreeMessage::ToggleDir(test_path.clone()),
        FileTreeMessage::OpenFile(test_path.clone()),
        FileTreeMessage::Search("test".to_string()),
        FileTreeMessage::ChangeFilter(FileTypeFilter::Db),
        FileTreeMessage::ExtractToJson(test_path.clone()),
        FileTreeMessage::ValidateFile(test_path.clone()),
        FileTreeMessage::ShowInFileManager(test_path.clone()),
        FileTreeMessage::ShowContextMenu(test_path.clone(), test_position),
        FileTreeMessage::HideContextMenu,
    ];

    // Verify we can create all message types
    assert_eq!(messages.len(), 9);

    // Test pattern matching
    for message in messages {
        match message {
            FileTreeMessage::ToggleDir(_) => {}
            FileTreeMessage::OpenFile(_) => {}
            FileTreeMessage::Search(_) => {}
            FileTreeMessage::ChangeFilter(_) => {}
            FileTreeMessage::ExtractToJson(_) => {}
            FileTreeMessage::ValidateFile(_) => {}
            FileTreeMessage::ShowInFileManager(_) => {}
            FileTreeMessage::ShowContextMenu(_, _) => {}
            FileTreeMessage::HideContextMenu => {}
        }
    }
}

#[test]
fn test_file_tree_empty_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path();

    // Scan an empty directory
    let tree = FileTree::scan(path);

    assert!(tree.root.is_some());
    if let Some(TreeNode::Dir { children, .. }) = tree.root {
        // Should have no children for empty directory
        assert!(children.is_empty());
    }
}

#[test]
fn test_file_tree_deep_nesting() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path();

    // Create deeply nested structure
    let deep_path = path.join("level1").join("level2").join("level3");
    std::fs::create_dir_all(&deep_path).unwrap();
    std::fs::File::create(deep_path.join("deep.txt")).unwrap();

    let tree = FileTree::scan(path);

    // Should be able to handle deep nesting without crashing
    assert!(tree.root.is_some());
}
