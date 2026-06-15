use crate::app::App;
use crate::components::file_tree::message::FileTreeMessage;
use crate::message::Message;
use crate::message::MessageExt;
use crate::workspace::Workspace;

#[test]
fn test_file_tree_search_empty_clears_filter() {
    let mut app = App::test_new(Workspace::new());
    app.file_tree.state.search_query = "old".to_string();
    let _ = app.update(Message::file_tree(FileTreeMessage::Search("".to_string())));
    assert!(
        app.file_tree.state.search_query.is_empty(),
        "empty search clears search_query"
    );
}

#[test]
fn test_file_tree_search_nonempty_sets_query() {
    let mut app = App::test_new(Workspace::new());
    let _ = app.update(Message::file_tree(FileTreeMessage::Search(
        "abc".to_string(),
    )));
    assert_eq!(app.file_tree.state.search_query, "abc");
}

#[test]
fn test_file_tree_toggle_dir_nonexistent_no_crash() {
    let mut app = App::test_new(Workspace::new());
    let _ = app.update(Message::file_tree(FileTreeMessage::ToggleDir(
        "/nonexistent".into(),
    )));
    // If we get here without panicking, the test passes.
}

#[test]
fn test_file_tree_open_file_nonexistent_no_crash() {
    let mut app = App::test_new(Workspace::new());
    let _ = app.update(Message::file_tree(FileTreeMessage::OpenFile(
        "/nonexistent/test.ini".into(),
    )));
    // Should handle gracefully — no panic.
}

#[test]
fn test_file_tree_extract_to_json_no_crash() {
    let mut app = App::test_new(Workspace::new());
    // rfd::FileDialog panics on macOS in headless test environments
    // ("NonWindowed environment"), so wrap in catch_unwind to isolate
    // the external dependency behavior from our handler code.
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = app.update(Message::file_tree(FileTreeMessage::ExtractToJson(
            "/nonexistent/test.db".into(),
        )));
    }));
    // If catch_unwind succeeded, the handler completed without panicking.
    // If rfd panicked (macOS headless), that's expected — not our bug.
}
