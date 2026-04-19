use std::path::{Path, PathBuf};

use crate::file_index_cache;

/// A node in the file tree.
#[derive(Debug, Clone)]
pub enum TreeNode {
    Dir {
        path: PathBuf,
        name: String,
        expanded: bool,
        children: Vec<TreeNode>,
    },
    File {
        path: PathBuf,
        name: String,
        icon: &'static str,
    },
}

impl TreeNode {
    /// Get the path of this node
    pub fn path(&self) -> Option<&Path> {
        match self {
            TreeNode::Dir { path, .. } => Some(path.as_path()),
            TreeNode::File { path, .. } => Some(path.as_path()),
        }
    }

    /// Add a directory child to this node
    pub fn add_directory_child(
        &mut self,
        file: &file_index_cache::CachedFileInfo,
        all_files: &[file_index_cache::CachedFileInfo],
    ) {
        if let TreeNode::Dir { children, .. } = self {
            // Skip system files/directories
            if file.name.starts_with('.') {
                return;
            }

            let mut dir_node = TreeNode::Dir {
                path: file.path.clone(),
                name: file.name.clone(),
                expanded: false,
                children: Vec::new(),
            };

            // Recursively add children if this directory has cached subdirectories
            for child_file in all_files {
                // Skip system files
                if child_file.name.starts_with('.') {
                    continue;
                }

                if child_file.is_directory && child_file.path.parent() == Some(&file.path) {
                    dir_node.add_directory_child(child_file, all_files);
                } else if !child_file.is_directory && child_file.path.parent() == Some(&file.path) {
                    dir_node.add_file_child(child_file);
                }
            }

            children.push(dir_node);
        }
    }

    /// Add a file child to this node
    pub fn add_file_child(&mut self, file: &file_index_cache::CachedFileInfo) {
        if let TreeNode::Dir { children, .. } = self {
            // Skip system files
            if file.name.starts_with('.') {
                return;
            }

            // Convert file type to static icon
            let icon = match file.file_type.as_str() {
                "db" => "🗃️",
                "ini" => "📄",
                "ref" => "📋",
                "scr" => "📜",
                "dlg" => "💬",
                "pgp" => "📝",
                "map" => "🗺️",
                "gtl" | "btl" => "🖼️",
                "spr" => "🎨",
                "snf" => "🔊",
                _ => "📎",
            };

            children.push(TreeNode::File {
                path: file.path.clone(),
                name: file.name.clone(),
                icon,
            });
        }
    }
}

/// Get the appropriate icon for a file based on its extension
pub fn file_icon(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("db") => "🗃️",
        Some("ini") => "📄",
        Some("ref") => "📋",
        Some("scr") => "📜",
        Some("dlg") => "💬",
        Some("pgp") => "📝",
        Some("map") => "🗺️",
        Some("gtl") | Some("btl") => "🖼️",
        Some("spr") => "🎨",
        Some("snf") => "🔊",
        _ => "📎",
    }
}
