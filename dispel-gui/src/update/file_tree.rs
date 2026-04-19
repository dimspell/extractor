// FileTree message handlers

use crate::app::App;
use crate::components::file_tree::message::FileTreeMessage;
use iced::Task;

pub fn handle(message: FileTreeMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        FileTreeMessage::ToggleDir(dir_path) => {
            app.file_tree.toggle(&dir_path);
            Task::none()
        }
        FileTreeMessage::OpenFile(file_path) => app.open_file_in_workspace(&file_path),
        FileTreeMessage::Search(query) => {
            // Use cache-aware search when query changes
            let cache_manager = app.state.file_index_cache_manager.clone();
            if let Some(ref manager) = cache_manager {
                if let Ok(Some(cache)) = manager.load_cache() {
                    if app.file_tree.should_use_cache(&Some(cache.clone())) {
                        app.file_tree = crate::components::file_tree::FileTree::build_from_cache(
                            &cache, &query,
                        );
                        app.file_tree.data.cache_manager = cache_manager;
                        return Task::none();
                    }
                }
            }
            app.file_tree.state.search_query = query.clone();
            app.file_tree.state.tree_filter =
                crate::components::file_tree::FileTreeFilter::new().with_search_query(query);
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
            app.state.show_in_file_manager(&file_path);
            Task::none()
        }
    }
}
