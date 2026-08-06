use crate::app::{App, AppMode};
use crate::message::Message;
use crate::message::startpage::StartPageMessage;
use crate::workspace::Workspace;
use std::path::PathBuf;

#[test]
fn test_path_input_changed_updates_input() {
    let mut app = App::test_new(Workspace::new());
    let task = app.update(Message::StartPage(StartPageMessage::PathInputChanged(
        "hello".into(),
    )));
    assert_eq!(app.start_page_input, "hello");
    assert_eq!(task.units(), 0);
}

#[test]
fn test_continue_nonexistent_path_noop() {
    let mut app = App::test_new(Workspace::new());
    app.start_page_input = "/proc/this_does_not_exist_12345xyz".to_string();
    let prev_mode = app.app_mode.clone();
    let task = app.update(Message::StartPage(StartPageMessage::Continue));
    assert_eq!(app.app_mode, prev_mode);
    assert_eq!(task.units(), 0);
}

#[test]
fn test_select_recent_path_nonexistent_noop() {
    let mut app = App::test_new(Workspace::new());
    let task = app.update(Message::StartPage(StartPageMessage::SelectRecentPath(
        PathBuf::from("/nonexistent/67890"),
    )));
    assert_eq!(task.units(), 0);
}

#[test]
fn test_back_to_start_sets_mode_and_restores_path() {
    let mut app = App::test_new(Workspace::new());
    app.state.workspace.game_path = Some(PathBuf::from("/some/path"));
    app.app_mode = AppMode::EditorMode;
    let task = app.update(Message::StartPage(StartPageMessage::BackToStart));
    assert_eq!(app.app_mode, AppMode::StartPage);
    assert_eq!(app.start_page_input, "/some/path");
    assert_eq!(task.units(), 0);
}

#[test]
fn test_back_to_start_no_game_path_does_not_change_input() {
    let mut app = App::test_new(Workspace::new());
    app.app_mode = AppMode::EditorMode;
    app.start_page_input = "existing".to_string();
    let task = app.update(Message::StartPage(StartPageMessage::BackToStart));
    assert_eq!(app.app_mode, AppMode::StartPage);
    assert_eq!(app.start_page_input, "existing");
    assert_eq!(task.units(), 0);
}
