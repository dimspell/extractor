use std::path::Path;

use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Fill, Font, Length, Padding};

use crate::components::file_tree::tree_node::TreeNode;
use crate::file_index_cache;
use crate::style;

use super::filter::{fuzzy_match, FileTreeError, FileTreeFilter, FileTreeResult};
use super::message::FileTreeMessage;
use crate::components::context_menu::ContextMenu;

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
    pub cancellation_token: Option<super::filter::CancellationToken>,
    pub notification_manager: super::filter::NotificationManager,
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

    /// Create a new cancellation token and start a scan operation
    pub fn start_scan_with_cancellation(&mut self) -> super::filter::CancellationToken {
        let token = super::filter::CancellationToken::new();
        self.state.cancellation_token = Some(token.clone());
        self.state.is_loading = true;
        token
    }

    /// Cancel the current scan operation
    pub fn cancel_scan(&mut self) {
        if let Some(token) = &self.state.cancellation_token {
            token.cancel();
        }
        self.state.is_loading = false;
        self.state.cancellation_token = None;
    }

    /// Check if current scan is cancelled
    pub fn is_cancelled(&self) -> bool {
        self.state
            .cancellation_token
            .as_ref()
            .map(|t| t.is_cancelled())
            .unwrap_or(false)
    }

    /// Handle a file tree error with recovery strategies and notifications
    pub fn handle_error(&mut self, error: &FileTreeError) -> Option<String> {
        self.set_loading(false);

        // Add notification for all errors
        self.add_error(error.user_message());

        // Apply recovery strategies
        match error {
            FileTreeError::CacheCorrupted => {
                // Automatically recover from cache corruption by clearing cache
                self.data.cache_manager = None;
                self.add_info("Cache has been cleared and will be rebuilt".to_string());
                Some(
                    "Cache was corrupted and has been cleared. Please try your operation again."
                        .to_string(),
                )
            }
            FileTreeError::PermissionDenied(_) => {
                self.add_warning(
                    "Some operations may be limited due to permission restrictions".to_string(),
                );
                Some("Permission denied. Please check file permissions and try again.".to_string())
            }
            FileTreeError::FileNotFound(_) => {
                Some("File not found. Please verify the file path.".to_string())
            }
            FileTreeError::Timeout => {
                self.add_warning("The operation took too long and was cancelled".to_string());
                Some("Operation timed out. Please try a simpler operation or check system resources.".to_string())
            }
            _ => {
                // For other errors, provide recovery suggestions if available
                let suggestions = error.recovery_suggestions();
                if !suggestions.is_empty() {
                    self.add_info(format!("Recovery suggestions: {}", suggestions.join(", ")));
                }
                Some(format!(
                    "An error occurred: {}. Please try again.",
                    error.user_message()
                ))
            }
        }
    }

    /// Clear cache and reset to default state
    pub fn reset_cache(&mut self) {
        self.data.cache_manager = None;
        self.state.tree_filter = FileTreeFilter::new();
    }

    /// Rebuild the file tree from scratch
    pub fn rebuild_tree(&mut self, _path: &Path) {
        self.reset_cache();
        // In a real implementation, this would trigger a rescan
        // For now, just reset the state
        self.data.root = None;
    }

    /// Add a notification to the notification manager
    pub fn add_notification(&mut self, notification: super::filter::FileTreeNotification) {
        self.state
            .notification_manager
            .add_notification(notification);
    }

    /// Get all notifications
    pub fn get_notifications(&self) -> Vec<super::filter::FileTreeNotification> {
        self.state.notification_manager.get_notifications()
    }

    /// Clear all notifications
    pub fn clear_notifications(&mut self) {
        self.state.notification_manager.clear_notifications();
    }

    /// Clear all error notifications
    pub fn clear_errors(&mut self) {
        self.state.notification_manager.clear_errors();
    }

    /// Add an error notification
    pub fn add_error(&mut self, message: String) {
        self.add_notification(super::filter::FileTreeNotification::error(message));
    }

    /// Add a warning notification
    pub fn add_warning(&mut self, message: String) {
        self.add_notification(super::filter::FileTreeNotification::warning(message));
    }

    /// Add an info notification
    pub fn add_info(&mut self, message: String) {
        self.add_notification(super::filter::FileTreeNotification::info(message));
    }

    /// Add a success notification
    pub fn add_success(&mut self, message: String) {
        self.add_notification(super::filter::FileTreeNotification::success(message));
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
    #[cfg(feature = "tokio")]
    pub async fn scan_async(
        path: &Path,
        cancellation_token: Option<&super::filter::CancellationToken>,
    ) -> super::filter::FileTreeResult<Self> {
        let root = scan_dir_async(path, 0, cancellation_token).await?;
        Ok(Self {
            data: FileTreeData {
                root,
                cache_manager: None,
            },
            state: FileTreeState::default(),
        })
    }

    /// Async version of scan_with_cache
    #[cfg(feature = "tokio")]
    pub async fn scan_with_cache_async(
        path: &Path,
        cache_manager: &Option<file_index_cache::FileIndexCacheManager>,
        cancellation_token: Option<&super::filter::CancellationToken>,
    ) -> super::filter::FileTreeResult<Self> {
        // Check for cancellation before starting
        if let Some(token) = cancellation_token {
            if token.is_cancelled() {
                return Err(FileTreeError::Cancelled);
            }
        }

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

    /// Toggle a directory's expanded state.
    pub fn toggle(&mut self, path: &Path) {
        if let Some(ref mut root) = self.data.root {
            toggle_node(root, path);
        }
    }

    /// Render the file tree.
    pub fn view(&self) -> Element<'_, FileTreeMessage> {
        let search_bar = text_input("Filter files...", &self.state.search_query)
            .on_input(FileTreeMessage::Search)
            .padding([4, 8])
            .size(11);

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

    /// Check if search should use cache.
    pub fn should_use_cache(&self, cache: &Option<file_index_cache::FileIndexCache>) -> bool {
        use std::time::{SystemTime, UNIX_EPOCH};

        cache.as_ref().is_some_and(|cache| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let cache_age = now.saturating_sub(cache.last_indexed);
            // 30 days in seconds (30 * 24 * 60 * 60)
            cache_age < 30 * 24 * 60 * 60
        })
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
                cancellation_token: None,
                is_loading: false,
                notification_manager: super::filter::NotificationManager::new(),
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

fn toggle_node(node: &mut TreeNode, path: &Path) {
    match node {
        TreeNode::Dir {
            path: dir_path,
            expanded,
            children,
            ..
        } => {
            if dir_path == path {
                *expanded = !*expanded;
                if *expanded && children.is_empty() {
                    *children = scan_children(dir_path);
                }
            } else {
                for child in children {
                    toggle_node(child, path);
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

            let show_children = *expanded || !tree_filter.search_query().is_empty();
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
            let search_path = path.to_string_lossy();
            if !tree_filter.matches_search(&search_path) {
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
                (
                    "Extract to JSON",
                    FileTreeMessage::ExtractToJson(path.clone()),
                ),
                ("Validate", FileTreeMessage::ValidateFile(path.clone())),
                (
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
#[cfg(feature = "tokio")]
async fn scan_dir_async(
    path: &Path,
    depth: usize,
    cancellation_token: Option<&super::filter::CancellationToken>,
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

    // Check for cancellation
    if let Some(token) = cancellation_token {
        if token.is_cancelled() {
            return Err(FileTreeError::Cancelled);
        }
    }

    // Check if path exists and is accessible
    match tokio::fs::metadata(path).await {
        Ok(_) => {}
        Err(e) => {
            log::debug!("Failed to access path {}: {}", path.display(), e);
            return Ok(None);
        }
    }

    if path.is_dir() {
        let children = Box::pin(scan_children_async(path, cancellation_token)).await?;
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

/// Async version of scan_children
#[cfg(feature = "tokio")]
async fn scan_children_async(
    path: &Path,
    cancellation_token: Option<&super::filter::CancellationToken>,
) -> FileTreeResult<Vec<TreeNode>> {
    let mut dirs = Vec::new();
    let mut files = Vec::new();

    // Check for cancellation before starting
    if let Some(token) = cancellation_token {
        if token.is_cancelled() {
            return Err(FileTreeError::Cancelled);
        }
    }

    match tokio::fs::read_dir(path).await {
        Ok(mut entries) => {
            while let Ok(entry_option) = entries.next_entry().await {
                // Check for cancellation periodically
                if let Some(token) = cancellation_token {
                    if token.is_cancelled() {
                        return Err(FileTreeError::Cancelled);
                    }
                }

                match entry_option {
                    Some(entry) => {
                        let p = entry.path();
                        if let Some(node) =
                            Box::pin(scan_dir_async(&p, 0, cancellation_token)).await?
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
