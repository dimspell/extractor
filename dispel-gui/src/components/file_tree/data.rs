use std::collections::HashSet;
use std::path::{Path, PathBuf};

use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Fill, Font, Length, Padding};

use crate::components::file_tree::tree_node::TreeNode;
use crate::indexation::file_index_cache;
use crate::style;

use super::filter::FileTreeResult;
use super::filter::{fuzzy_match, FileTreeFilter};
use super::message::FileTreeMessage;
use gui_widgets::components::context_menu::{ContextMenu, Entry};

/// File tree data structure (pure data representation).
#[derive(Debug, Clone, Default)]
pub struct FileTreeData {
    pub root: Option<TreeNode>,
    pub cache_manager: Option<file_index_cache::FileIndexCacheManager>,
}

/// File tree UI state.
#[derive(Debug, Clone, Default)]
pub struct FileTreeState {
    pub search_query: String,
    pub tree_filter: FileTreeFilter,
    pub is_loading: bool,
    pub loading_dirs: HashSet<PathBuf>,
}

/// File tree widget state (combines data and UI state for backward compatibility).
#[derive(Debug, Clone, Default)]
pub struct FileTree {
    pub data: FileTreeData,
    pub state: FileTreeState,
}

impl FileTree {
    /// Set loading state
    pub fn set_loading(&mut self, is_loading: bool) {
        self.state.is_loading = is_loading;
    }

    /// Check if currently loading
    pub fn is_loading(&self) -> bool {
        self.state.is_loading
    }

}

impl FileTree {
    /// Scan a directory and build the tree.
    pub fn scan(path: &Path) -> Self {
        let root = scan_dir(path, 0);
        Self {
            data: FileTreeData {
                root,
                cache_manager: None,
            },
            state: FileTreeState::default(),
        }
    }

    /// Scan a directory using cache if available, otherwise fall back to regular scanning.
    pub fn scan_with_cache(
        path: &Path,
        cache_manager: &Option<file_index_cache::FileIndexCacheManager>,
    ) -> Self {
        if let Some(ref manager) = cache_manager {
            if let Ok(Some(cache)) = manager.load_cache() {
                if file_index_cache::CacheValidator::validate_cache(&cache, path) {
                    return Self {
                        data: FileTreeData {
                            root: Some(Self::cache_to_tree_node(&cache)),
                            cache_manager: cache_manager.clone(),
                        },
                        state: FileTreeState::default(),
                    };
                }
            }
        }
        Self::scan(path)
    }

    /// Convert cache data to tree node format.
    fn cache_to_tree_node(cache: &file_index_cache::FileIndexCache) -> TreeNode {
        let mut root_dir = TreeNode::Dir {
            path: cache.game_path.clone(),
            name: cache
                .game_path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Game Files".to_string()),
            expanded: true,
            children: Vec::new(),
        };

        for file in &cache.files {
            if file.is_directory && file.path.parent() == Some(&cache.game_path) {
                // Only add directories that are direct children of the root directory
                root_dir.add_directory_child(file, &cache.files);
            } else if !file.is_directory && file.path.parent() == Some(&cache.game_path) {
                // Only add files that are direct children of the root directory
                root_dir.add_file_child(file);
            }
            // Files and directories in subdirectories will be added by add_directory_child
        }

        root_dir
    }

    /// Async version of scan
    pub async fn scan_async(
        path: &Path,
    ) -> super::filter::FileTreeResult<Self> {
        let root = scan_dir_async(path, 0).await?;
        Ok(Self {
            data: FileTreeData {
                root,
                cache_manager: None,
            },
            state: FileTreeState::default(),
        })
    }

    /// Async version of scan_with_cache
    pub async fn scan_with_cache_async(
        path: &Path,
        cache_manager: &Option<file_index_cache::FileIndexCacheManager>,
    ) -> super::filter::FileTreeResult<Self> {
        if let Some(ref manager) = cache_manager {
            if let Ok(Some(cache)) = manager.load_cache() {
                if file_index_cache::CacheValidator::validate_cache(&cache, path) {
                    return Ok(Self {
                        data: FileTreeData {
                            root: Some(Self::cache_to_tree_node(&cache)),
                            cache_manager: cache_manager.clone(),
                        },
                        state: FileTreeState::default(),
                    });
                }
            }
        }
        Ok(Self::scan(path))
    }

    /// Toggle a directory's expanded state (sync). Returns true if children
    /// need to be loaded asynchronously (was just expanded, no children present).
    pub fn toggle_expanded(&mut self, path: &Path) -> bool {
        if let Some(ref mut root) = self.data.root {
            toggle_node_expanded_only(root, path)
        } else {
            false
        }
    }

    /// Set children of a directory node (called from ToggleDirComplete handler).
    pub fn set_children(&mut self, path: &Path, children: Vec<TreeNode>) {
        if let Some(ref mut root) = self.data.root {
            set_node_children(root, path, children);
        }
    }

    /// Render the file tree.
    pub fn view(&self) -> Element<'_, FileTreeMessage> {
        let search_bar = text_input("Filter files...", &self.state.search_query)
            .on_input(FileTreeMessage::Search)
            .padding([4, 8])
            .size(11)
            .accessible_label("Filter file tree");

        let header = container(search_bar).padding([6, 4]);

        let tree_content: Element<'_, FileTreeMessage> = match &self.data.root {
            Some(node) => render_node(node, &self.state.tree_filter, 0)
                .map(|e| column![e].into())
                .unwrap_or_else(|| {
                    column![text("No matching files").size(11).style(style::subtle_text)]
                        .padding([4, 8])
                        .into()
                }),
            None => column![text("No game path set").size(11).style(style::subtle_text)]
                .padding([4, 8])
                .into(),
        };

        column![header, scrollable(tree_content).height(Length::Fill)]
            .spacing(0)
            .height(Fill)
            .into()
    }

    /// Build tree from cache data for faster loading.
    pub fn build_from_cache(cache: &file_index_cache::FileIndexCache, query: &str) -> Self {
        let game_path = cache.game_path.clone();
        let files = cache.files.clone();

        // Build proper hierarchy using the same logic as cache_to_tree_node
        let mut root_dir = TreeNode::Dir {
            path: game_path.clone(),
            name: game_path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Game Files".to_string()),
            expanded: true,
            children: Vec::new(),
        };

        // Create tree filter for this search
        let tree_filter = FileTreeFilter::new().with_search_query(query.to_string());

        // Filter files based on query - use full path so subdirectories are searchable
        let filtered_files: Vec<_> = files
            .iter()
            .filter(|f| {
                let search_path = f.path.to_string_lossy();
                tree_filter.matches_search(&search_path)
            })
            .cloned()
            .collect();

        // Skip system files
        let filtered_files: Vec<_> = filtered_files
            .into_iter()
            .filter(|f| !f.name.starts_with('.'))
            .collect();

        // When filtering, we need to include parent directories of matching files
        // so the tree structure is preserved (e.g., if we find NpcInGame/Face1.spr,
        // we need to show the NpcInGame directory even if it doesn't match the query)
        let mut files_to_show: Vec<_> = filtered_files.clone();

        // Add parent directories of all filtered files
        for file in &filtered_files {
            let mut parent = file.path.parent();
            while let Some(p) = parent {
                if p == game_path {
                    break;
                }
                // Check if we already have this directory
                if !files_to_show.iter().any(|f| f.path == p) {
                    // Find the directory info from original files list
                    if let Some(dir_info) = files.iter().find(|f| f.path == p && f.is_directory) {
                        files_to_show.push(dir_info.clone());
                    }
                }
                parent = p.parent();
            }
        }

        // Build hierarchy
        for file in &files_to_show {
            if file.is_directory && file.path.parent() == Some(&game_path) {
                // Only add directories that are direct children of the root directory
                root_dir.add_directory_child(file, &files_to_show);
            } else if !file.is_directory && file.path.parent() == Some(&game_path) {
                // Only add files that are direct children of the root directory
                root_dir.add_file_child(file);
            }
            // Files and directories in subdirectories will be added by add_directory_child
        }

        let root = Some(root_dir);

        Self {
            data: FileTreeData {
                root,
                cache_manager: None,
            },
            state: FileTreeState {
                search_query: query.to_string(),
                tree_filter: FileTreeFilter::new().with_search_query(query.to_string()),
                is_loading: false,
                loading_dirs: HashSet::new(),
            },
        }
    }
}

fn scan_dir(path: &Path, depth: usize) -> Option<TreeNode> {
    // Skip system files like .DS_STORE
    if let Some(name) = path.file_name() {
        if name.to_string_lossy().starts_with('.') {
            return None;
        }
    }

    let name = path.file_name()?.to_string_lossy().to_string();

    if path.is_dir() {
        Some(TreeNode::Dir {
            path: path.to_path_buf(),
            name,
            expanded: depth == 0,
            children: scan_children(path),
        })
    } else {
        let icon = super::tree_node::file_icon(path);
        Some(TreeNode::File {
            path: path.to_path_buf(),
            name,
            icon,
        })
    }
}

fn scan_children(path: &Path) -> Vec<TreeNode> {
    let mut dirs = Vec::new();
    let mut files = Vec::new();

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Some(node) = scan_dir(&p, 0) {
                // Reset depth to 0 for proper nesting
                match &node {
                    TreeNode::Dir { .. } => dirs.push(node),
                    TreeNode::File { .. } => files.push(node),
                }
            }
        }
    }

    dirs.sort_by_key(|n| match n {
        TreeNode::Dir { name, .. } | TreeNode::File { name, .. } => name.to_lowercase(),
    });
    files.sort_by_key(|n| match n {
        TreeNode::Dir { name, .. } | TreeNode::File { name, .. } => name.to_lowercase(),
    });

    dirs.extend(files);
    dirs
}

/// Flip expanded state only — does NOT scan children.
fn toggle_node_expanded_only(node: &mut TreeNode, path: &Path) -> bool {
    match node {
        TreeNode::Dir {
            path: dir_path,
            expanded,
            children,
            ..
        } => {
            if dir_path == path {
                let was_expanded = *expanded;
                *expanded = !*expanded;
                // Return true if we just expanded and children haven't been loaded yet
                *expanded && children.is_empty() && !was_expanded
            } else {
                for child in children {
                    if toggle_node_expanded_only(child, path) {
                        return true;
                    }
                }
                false
            }
        }
        TreeNode::File { .. } => false,
    }
}

/// Set children of a specific directory node.
fn set_node_children(node: &mut TreeNode, path: &Path, children: Vec<TreeNode>) {
    match node {
        TreeNode::Dir {
            path: dir_path,
            children: ch,
            ..
        } => {
            if dir_path == path {
                *ch = children;
            } else {
                for child in ch {
                    set_node_children(child, path, children.clone());
                }
            }
        }
        TreeNode::File { .. } => {}
    }
}

/// Render a tree node as an Element. Returns None if the node should be hidden
/// (file doesn't match current query/filter). Directories are always rendered.
fn render_node<'a>(
    node: &'a TreeNode,
    tree_filter: &'a FileTreeFilter,
    depth: usize,
) -> Option<Element<'a, FileTreeMessage>> {
    // Each depth level adds 14px of left padding.
    let left_pad = (6 + depth * 12) as f32;

    match node {
        TreeNode::Dir {
            path,
            name,
            expanded,
            children,
        } => {
            let caret = if *expanded { "▼" } else { "▶" };

            let header = button(
                row![
                    text(caret).size(9).style(style::subtle_text),
                    text(name).size(12),
                ]
                .spacing(5)
                .align_y(iced::Alignment::Center),
            )
            .on_press(FileTreeMessage::ToggleDir(path.clone()))
            .width(Fill)
            .style(style::tree_dir_row)
            .padding(Padding {
                top: 3.0,
                right: 4.0,
                bottom: 3.0,
                left: left_pad,
            });

            let mut content = column![header].spacing(0);

            let show_children = *expanded || tree_filter.matching_paths().is_some();
            if show_children {
                for child in children {
                    if let Some(child_element) = render_node(child, tree_filter, depth + 1) {
                        content = content.push(child_element);
                    }
                }
            }

            Some(content.into())
        }
        TreeNode::File { path, name, icon } => {
            if !tree_filter.is_path_matching(path) {
                return None;
            }

            let name_element = create_highlighted_text(name, tree_filter.search_query());

            let file_btn = button(
                row![text(*icon).size(10), name_element]
                    .spacing(5)
                    .align_y(iced::Alignment::Center),
            )
            .on_press(FileTreeMessage::OpenFile(path.clone()))
            .width(Fill)
            .style(style::tree_file_row)
            .padding(Padding {
                top: 2.0,
                right: 4.0,
                bottom: 2.0,
                left: left_pad + 12.0,
            });

            let entries = vec![
                Entry::item("Open as Hex", FileTreeMessage::OpenAsHex(path.clone())),
                Entry::separator(),
                Entry::item(
                    "Extract to JSON",
                    FileTreeMessage::ExtractToJson(path.clone()),
                ),
                Entry::item("Validate", FileTreeMessage::ValidateFile(path.clone())),
                Entry::item(
                    "Show in File Manager",
                    FileTreeMessage::ShowInFileManager(path.clone()),
                ),
            ];

            Some(ContextMenu::new(file_btn, entries).into())
        }
    }
}

/// Build a row with fuzzy-matched characters highlighted.
fn create_highlighted_text<'a>(name: &'a str, query: &str) -> Element<'a, FileTreeMessage> {
    let Some(match_indices) = fuzzy_match(query, name) else {
        return text(name).size(11).font(Font::MONOSPACE).into();
    };

    if match_indices.is_empty() {
        return text(name).size(11).font(Font::MONOSPACE).into();
    }

    let chars: Vec<char> = name.chars().collect();
    let mut r = row![].spacing(0);
    let mut segment = String::new();
    let mut in_match = false;

    let mut mi = 0; // index into match_indices
    for (ci, ch) in chars.iter().enumerate() {
        let is_matched = mi < match_indices.len() && match_indices[mi] == ci;
        if is_matched {
            mi += 1;
        }

        if is_matched != in_match && !segment.is_empty() {
            if in_match {
                r = r.push(
                    text(segment.clone())
                        .size(11)
                        .font(Font::MONOSPACE)
                        .style(style::primary_text),
                );
            } else {
                r = r.push(text(segment.clone()).size(11).font(Font::MONOSPACE));
            }
            segment.clear();
        }
        in_match = is_matched;
        segment.push(*ch);
    }

    if !segment.is_empty() {
        if in_match {
            r = r.push(
                text(segment)
                    .size(11)
                    .font(Font::MONOSPACE)
                    .style(style::primary_text),
            );
        } else {
            r = r.push(text(segment).size(11).font(Font::MONOSPACE));
        }
    }

    r.into()
}

/// Async version of scan_dir
async fn scan_dir_async(
    path: &Path,
    depth: usize,
) -> FileTreeResult<Option<TreeNode>> {
    // Skip system files like .DS_STORE
    if let Some(name) = path.file_name() {
        if name.to_string_lossy().starts_with('.') {
            return Ok(None);
        }
    }

    let name = match path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => {
            log::warn!("Path has no file name: {}", path.display());
            return Ok(None);
        }
    };

    // Check if path exists and is accessible
    match tokio::fs::metadata(path).await {
        Ok(_) => {}
        Err(e) => {
            log::debug!("Failed to access path {}: {}", path.display(), e);
            return Ok(None);
        }
    }

    if path.is_dir() {
        let children = Box::pin(scan_children_async(path)).await?;
        Ok(Some(TreeNode::Dir {
            path: path.to_path_buf(),
            name,
            expanded: depth == 0,
            children,
        }))
    } else {
        let icon = super::tree_node::file_icon(path);
        Ok(Some(TreeNode::File {
            path: path.to_path_buf(),
            name,
            icon,
        }))
    }
}

/// Async version of scan_children (public, used by the ToggleDir handler).
pub async fn scan_children_async(
    path: &Path,
) -> FileTreeResult<Vec<TreeNode>> {
    let mut dirs = Vec::new();
    let mut files = Vec::new();

    match tokio::fs::read_dir(path).await {
        Ok(mut entries) => {
            while let Ok(entry_option) = entries.next_entry().await {
                match entry_option {
                    Some(entry) => {
                        let p = entry.path();
                        if let Some(node) =
                            Box::pin(scan_dir_async(&p, 0)).await?
                        {
                            // Reset depth to 0 for proper nesting
                            match &node {
                                TreeNode::Dir { .. } => dirs.push(node),
                                TreeNode::File { .. } => files.push(node),
                            }
                        }
                    }
                    None => break, // End of directory entries
                }
            }
        }
        Err(e) => {
            log::error!("Failed to read directory {}: {}", path.display(), e);
            return Ok(Vec::new()); // Return empty vec on error, but don't fail the entire operation
        }
    }

    dirs.sort_by_key(|n| match n {
        TreeNode::Dir { name, .. } | TreeNode::File { name, .. } => name.to_lowercase(),
    });
    files.sort_by_key(|n| match n {
        TreeNode::Dir { name, .. } | TreeNode::File { name, .. } => name.to_lowercase(),
    });

    dirs.extend(files);
    Ok(dirs)
}
