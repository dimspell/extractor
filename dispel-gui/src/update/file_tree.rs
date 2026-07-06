// FileTree message handlers

use crate::app::App;
use crate::components::file_tree::data::scan_children_async;
use crate::components::file_tree::message::FileTreeMessage;
use crate::components::file_tree::tree_node::TreeNode;
use crate::message::MessageExt;
use iced::Task;
use std::path::PathBuf;
use std::time::Duration;

pub fn handle(message: FileTreeMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        FileTreeMessage::ToggleDir(dir_path) => {
            let needs_load = app.file_tree.toggle_expanded(&dir_path);
            if needs_load {
                // Avoid duplicate loads
                if !app.file_tree.state.loading_dirs.insert(dir_path.clone()) {
                    return Task::none(); // already loading this directory
                }
                return Task::perform(
                    async move {
                        let result = scan_children_async(&dir_path).await;
                        (dir_path, result.unwrap_or_default())
                    },
                    |(path, children): (PathBuf, Vec<TreeNode>)| {
                        crate::message::Message::file_tree(FileTreeMessage::ToggleDirComplete(
                            path, children,
                        ))
                    },
                );
            }
            Task::none()
        }
        FileTreeMessage::ToggleDirComplete(dir_path, children) => {
            app.file_tree.state.loading_dirs.remove(&dir_path);
            app.file_tree.set_children(&dir_path, children);
            Task::none()
        }
        FileTreeMessage::OpenFile(file_path) => app.open_file_in_workspace(&file_path),
        FileTreeMessage::OpenAsHex(file_path) => app.open_file_in_workspace_as_hex(&file_path),
        FileTreeMessage::Search(query) => {
            // Always update the raw input immediately for responsive text field.
            app.file_tree.state.search_query = query.clone();

            if query.is_empty() {
                // Clear filter immediately — no debounce needed.
                app.file_tree.state.tree_filter =
                    crate::components::file_tree::FileTreeFilter::new().with_search_query(query);
                return Task::none();
            }

            // Debounce: wait 150ms of idle time before applying the filter.
            // If the user types more before the delay elapses, the task checks
            // whether `search_query` still matches — older tasks are silently dropped.
            Task::perform(
                async move {
                    tokio::time::sleep(Duration::from_millis(150)).await;
                    query
                },
                |q| crate::message::Message::file_tree(FileTreeMessage::ApplyDebouncedSearch(q)),
            )
        }
        FileTreeMessage::ApplyDebouncedSearch(query) => {
            // Only apply if the user hasn't typed more since this task was spawned.
            if app.file_tree.state.search_query == query {
                app.file_tree.state.tree_filter =
                    crate::components::file_tree::FileTreeFilter::new()
                        .with_search_query(query.clone());
                // Pre-compute matching paths so view() does O(1) lookups per file.
                app.file_tree
                    .state
                    .tree_filter
                    .build_matching_paths(app.file_tree.data.root.as_ref());
            }
            Task::none()
        }
        FileTreeMessage::ExtractToJson(file_path) => {
            app.state.extract_file_to_json(&file_path);
            Task::none()
        }
        FileTreeMessage::ValidateFile(file_path) => {
            app.state.validate_file(&file_path);
            Task::none()
        }
        FileTreeMessage::ShowInFileManager(file_path) => {
            crate::platform::open_in_file_manager(&file_path);
            Task::none()
        }
    }
}
