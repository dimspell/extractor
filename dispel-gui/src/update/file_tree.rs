// FileTree message handlers

use crate::app::App;
use crate::components::file_tree::message::FileTreeMessage;
use crate::message::MessageExt;
use iced::Task;
use std::time::Duration;

pub fn handle(message: FileTreeMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        FileTreeMessage::ToggleDir(dir_path) => {
            app.file_tree.toggle(&dir_path);
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
                    crate::components::file_tree::FileTreeFilter::new().with_search_query(query);
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
