// Integration tests for complete message flows
use dispel_gui::app::App;
use dispel_gui::message::*;
use dispel_gui::update;
use iced::Task;

#[test]
fn test_weapon_editor_flow() {
    let mut app = App::default();

    // Test weapon scan flow
    let scan_msg = Message::weapon(WeaponEditorMessage::ScanWeapons);
    let task = app.update(scan_msg);

    // Should return a task (async operation)
    assert!(matches!(task, Task::perform(_, _)));
}

#[test]
fn test_monster_editor_flow() {
    let mut app = App::default();

    // Test monster selection flow
    let select_msg = Message::Editor(EditorMessage::Monster(MonsterEditorMessage::SelectMonster(
        0,
    )));
    let task = app.update(select_msg);

    // Should complete immediately
    assert!(matches!(task, Task::none()));
}

#[test]
fn test_workspace_toggle() {
    let mut app = App::default();

    // Test workspace toggle
    let toggle_msg = Message::Workspace(WorkspaceMessage::ToggleWorkspaceMode);
    let task = app.update(toggle_msg);

    assert!(matches!(task, Task::none()));
}

#[test]
fn test_message_routing_completeness() {
    let mut app = App::default();

    // Test that all message types can be routed without panic
    let test_messages = vec![
        Message::weapon(WeaponEditorMessage::ScanWeapons),
        Message::Editor(EditorMessage::Monster(MonsterEditorMessage::SelectMonster(
            0,
        ))),
        Message::Workspace(WorkspaceMessage::ToggleWorkspaceMode),
        Message::FileTree(FileTreeMessage::Browse),
        Message::Viewer(ViewerMessage::DbPathChanged("test".to_string())),
        Message::System(SystemMessage::CloseRequested),
    ];

    for msg in test_messages {
        let _ = app.update(msg);
        // If we get here without panic, routing worked
    }
}
