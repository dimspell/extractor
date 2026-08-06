use std::path::{Path, PathBuf};

use lucide_icons::Icon;

use crate::indexation::file_index_cache;

/// Payload for a file-tree [`TreeNode`].
///
/// Replaces the old `TreeNode` enum — the generic `TreeNode<T>` lives
/// in `gui-widgets` and this struct is the `T` for the file-tree use-case.
#[derive(Debug, Clone)]
pub struct GameFileNode {
    pub path: PathBuf,
    pub name: String,
    pub icon: Icon,
    pub is_dir: bool,
}

impl GameFileNode {
    pub fn dir(path: PathBuf, name: String) -> Self {
        GameFileNode {
            path,
            name,
            icon: Icon::Folder,
            is_dir: true,
        }
    }

    pub fn file(path: PathBuf, name: String, icon: Icon) -> Self {
        GameFileNode {
            path,
            name,
            icon,
            is_dir: false,
        }
    }
}

/// Build a `TreeNode<GameFileNode>` for the given cache directory entry,
/// recursively adding descendants found in `all_files`.
///
/// Returns `None` when the entry should be skipped (dotfiles).
pub fn add_cache_directory_child(
    file: &file_index_cache::CachedFileInfo,
    all_files: &[file_index_cache::CachedFileInfo],
) -> Option<gui_widgets::components::TreeNode<GameFileNode>> {
    if file.name.starts_with('.') {
        return None;
    }

    let mut dir_node = gui_widgets::components::TreeNode::branch(
        GameFileNode::dir(file.path.clone(), file.name.clone()),
        Vec::new(),
    );

    for child_file in all_files {
        if child_file.name.starts_with('.') {
            continue;
        }

        if child_file.is_directory && child_file.path.parent() == Some(&file.path) {
            if let Some(child) = add_cache_directory_child(child_file, all_files) {
                dir_node.children.push(child);
            }
        } else if !child_file.is_directory
            && child_file.path.parent() == Some(&file.path)
            && let Some(child) = add_cache_file_child(child_file)
        {
            dir_node.children.push(child);
        }
    }

    Some(dir_node)
}

/// Build a leaf `TreeNode<GameFileNode>` from a cached file entry.
pub fn add_cache_file_child(
    file: &file_index_cache::CachedFileInfo,
) -> Option<gui_widgets::components::TreeNode<GameFileNode>> {
    if file.name.starts_with('.') {
        return None;
    }

    let icon = match file.file_type.as_str() {
        "db" => Icon::Database,
        "ini" => Icon::FileText,
        "ref" => Icon::ClipboardList,
        "scr" => Icon::ScrollText,
        "dlg" => Icon::MessageSquare,
        "pgp" => Icon::FileEdit,
        "map" => Icon::Map,
        "gtl" | "btl" => Icon::Image,
        "spr" => Icon::Palette,
        "snf" => Icon::Music,
        _ => Icon::Paperclip,
    };

    Some(gui_widgets::components::TreeNode::leaf(GameFileNode::file(
        file.path.clone(),
        file.name.clone(),
        icon,
    )))
}

/// Get the appropriate icon for a file based on its extension.
pub fn file_icon(path: &Path) -> Icon {
    match path.extension().and_then(|e| e.to_str()) {
        Some("db") => Icon::Database,
        Some("ini") => Icon::FileText,
        Some("ref") => Icon::ClipboardList,
        Some("scr") => Icon::ScrollText,
        Some("dlg") => Icon::MessageSquare,
        Some("pgp") => Icon::FileEdit,
        Some("map") => Icon::Map,
        Some("gtl") | Some("btl") => Icon::Image,
        Some("spr") => Icon::Palette,
        Some("snf") => Icon::Music,
        _ => Icon::Paperclip,
    }
}
