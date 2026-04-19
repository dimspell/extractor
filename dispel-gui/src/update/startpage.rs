use crate::app::{App, AppMode};
use crate::message::startpage::StartPageMessage;
use crate::message::{system::SystemMessage, Message};
use crate::utils::browse_folder;
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: StartPageMessage, app: &mut App) -> Task<Message> {
    match message {
        StartPageMessage::PathInputChanged(s) => {
            app.start_page_input = s;
            Task::none()
        }
        StartPageMessage::Browse => browse_folder("start_page_path"),
        StartPageMessage::Continue => {
            let path = PathBuf::from(&app.start_page_input);
            if !path.exists() {
                return Task::none();
            }
            enter_editor_mode(app, path)
        }
        StartPageMessage::SelectRecentPath(path) => {
            app.start_page_input = path.to_string_lossy().to_string();
            Task::none()
        }
        StartPageMessage::BackToStart => {
            app.app_mode = AppMode::StartPage;
            if let Some(ref gp) = app.state.workspace.game_path {
                app.start_page_input = gp.to_string_lossy().to_string();
            }
            Task::none()
        }
    }
}

pub fn enter_editor_mode(app: &mut App, path: PathBuf) -> Task<Message> {
    // Update recent game paths (max 5)
    let recent = &mut app.state.workspace.recent_game_paths;
    recent.retain(|p| p != &path);
    recent.insert(0, path.clone());
    recent.truncate(5);

    // Set game path
    app.state.workspace.game_path = Some(path.clone());
    app.state.shared_game_path = path.to_string_lossy().to_string();

    // Scan file tree with existing cache
    let cache_mgr = app.state.file_index_cache_manager.clone();
    app.file_tree = crate::components::file_tree::FileTree::scan_with_cache(&path, &cache_mgr);

    // Switch to editor mode and start indexing
    app.app_mode = AppMode::EditorMode;
    app.is_indexing = true;

    // Save workspace
    app.save_workspace();

    // Rebuild search index and file cache
    app.search_index.clear();
    app.search_index.game_path = Some(path.to_string_lossy().to_string());
    let gp = path.clone();
    let gp_cache = path.clone();

    let search_task = Task::perform(
        async move { crate::search_index::build_index(&gp).await },
        |index| Message::System(SystemMessage::IndexLoaded(Ok(index))),
    );

    if let Some(ref cache_manager) = app.state.file_index_cache_manager {
        let cache_manager = cache_manager.clone();
        Task::batch([
            search_task,
            Task::perform(
                async move {
                    let svc = crate::indexation_service::IndexationService::new(cache_manager);
                    svc.start_indexation_with_fallback(gp_cache).await
                },
                |result| match result {
                    Ok(cache) => Message::System(SystemMessage::CacheIndexationComplete(cache)),
                    Err(_) => Message::System(SystemMessage::CacheIndexationFailed),
                },
            ),
        ])
    } else {
        search_task
    }
}
