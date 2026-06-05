#[cfg(test)]
mod message_routing_tests {
    use crate::app::App;
    use crate::message::Message;
    use crate::message::system::SystemMessage;
    use crate::message::workspace::WorkspaceMessage;
    use crate::tests::app_with_tab;
    use crate::workspace::{EditorType, Workspace};
    use std::path::PathBuf;

    #[test]
    fn test_hex_editor_message_when_no_state_does_not_panic() {
        let mut app = app_with_tab(EditorType::HexEditor);

        use crate::message::editor::EditorMessage;
        use hexedit::selection::NavDir;
        use hexedit::HexEditorMessage;
        // Route through the public update API — should not panic
        let task = app.update(Message::Editor(EditorMessage::HexEditor(
            HexEditorMessage::Nav {
                dir: NavDir::Down,
                extend: false,
            },
        )));
        assert_eq!(
            task.units(), 0,
            "Should return Task::none() when no hex editor state"
        );
    }

    #[test]
    fn test_open_tool_tab_creates_workspace_tab() {
        let mut app = App::test_new(Workspace::new());

        use crate::message::workspace::WorkspaceMessage;
        let task = app.update(Message::Workspace(WorkspaceMessage::OpenToolTab(
            EditorType::DbViewer,
        )));

        assert_eq!(app.state.workspace.tabs.len(), 1);
        assert_eq!(
            app.state.workspace.tabs[0].editor_type,
            EditorType::DbViewer
        );
        assert_eq!(app.state.workspace.tabs[0].label, "DB Viewer");

        let _ = task;
    }

    #[test]
    fn test_open_tool_tab_without_game_path_works() {
        let mut app = App::test_new(Workspace::new());

        use crate::message::workspace::WorkspaceMessage;
        let task = app.update(Message::Workspace(WorkspaceMessage::OpenToolTab(
            EditorType::ChestEditor,
        )));

        assert_eq!(app.state.workspace.tabs.len(), 1);
        assert_eq!(
            app.state.workspace.tabs[0].editor_type,
            EditorType::ChestEditor
        );

        let _ = task;
    }

    #[test]
    fn test_clear_workspace_system_message_is_idempotent() {
        let mut app = App::test_new(Workspace::new());

        app.state.workspace.open("test".into(), None);
        assert_eq!(app.state.workspace.tabs.len(), 1);

        // First clear
        let task1 = app.update(Message::System(
            crate::message::system::SystemMessage::ClearWorkspace,
        ));
        let _ = task1;
        assert_eq!(app.state.workspace.tabs.len(), 0);

        // Second clear — must not panic and must keep state clean
        let task2 = app.update(Message::System(
            crate::message::system::SystemMessage::ClearWorkspace,
        ));
        let _ = task2;
        assert_eq!(app.state.workspace.tabs.len(), 0);
    }

    #[test]
    fn test_open_file_in_workspace_dialogue_script_creates_state() {
        let mut app = App::test_new(Workspace::new());

        let path = PathBuf::from("/game/scene.dlg");
        let _task = app.open_file_in_workspace(&path);

        assert!(app.state.workspace.tabs.len() >= 1);

        let tab = app.state.workspace.active().unwrap();
        assert_eq!(tab.editor_type, EditorType::DialogueScriptEditor);
    }

    #[test]
    fn test_open_file_in_workspace_creates_tab_and_tracks_recent() {
        let mut app = App::test_new(Workspace::new());

        let path = PathBuf::from("/game/Monster.db");
        let _task = app.open_file_in_workspace(&path);

        let tab = app.state.workspace.active().unwrap();
        assert_eq!(tab.editor_type, EditorType::MonsterEditor);

        assert!(!app.state.recent_files.is_empty());
        assert_eq!(app.state.recent_files[0], path);
    }

    #[test]
    fn test_open_file_in_workspace_hex_editor_for_unknown_extension() {
        let mut app = App::test_new(Workspace::new());

        let path = PathBuf::from("/game/random.xyz");
        let _task = app.open_file_in_workspace(&path);

        let tab = app.state.workspace.active().unwrap();
        assert_eq!(
            tab.editor_type,
            EditorType::HexEditor,
            "Unknown extension should fall back to HexEditor"
        );
    }

    #[test]
    fn test_open_same_file_reactivates_existing_tab() {
        let mut app = App::test_new(Workspace::new());

        let path = PathBuf::from("/game/weaponItem.db");
        let _task1 = app.open_file_in_workspace(&path);

        let _task2 = app.open_file_in_workspace(&path);

        // Should still be just one tab (reactivated), not two
        assert_eq!(app.state.workspace.tabs.len(), 1);
    }

    #[test]
    fn test_track_recent_files_limit() {
        let mut app = App::test_new(Workspace::new());

        // Add 15 files (should be capped to 10)
        for i in 0..15 {
            app.track_recent_file(&PathBuf::from(format!("/game/file{}.db", i)));
        }

        assert_eq!(app.state.recent_files.len(), 10);
        assert_eq!(
            app.state.recent_files[0],
            PathBuf::from("/game/file14.db")
        );
    }

    #[test]
    fn test_track_recent_file_dedup() {
        let mut app = App::test_new(Workspace::new());

        app.track_recent_file(&PathBuf::from("/game/weaponItem.db"));
        app.track_recent_file(&PathBuf::from("/game/monster.db"));
        app.track_recent_file(&PathBuf::from("/game/weaponItem.db"));

        assert_eq!(app.state.recent_files.len(), 2);
        assert_eq!(
            app.state.recent_files[0],
            PathBuf::from("/game/weaponItem.db")
        );
    }

    #[test]
    fn test_editor_type_from_path_round_trips_all_db_types() {
        let cases = vec![
            ("weaponItem.db", EditorType::WeaponEditor),
            ("Monster.db", EditorType::MonsterEditor),
            ("HealItem.db", EditorType::HealItemEditor),
            ("MiscItem.db", EditorType::MiscItemEditor),
            ("EditItem.db", EditorType::EditItemEditor),
            ("EventItem.db", EditorType::EventItemEditor),
            ("Store.db", EditorType::StoreEditor),
            ("Magic.db", EditorType::MagicEditor),
            ("ChData.db", EditorType::ChDataEditor),
            ("PrtLevel.db", EditorType::PartyLevelDbEditor),
            ("PrtIni.db", EditorType::PartyIniEditor),
        ];
        for (filename, expected) in cases {
            let result = EditorType::from_path(PathBuf::from(filename).as_path());
            assert_eq!(result, expected, "failed for {filename}");
        }
    }

    #[test]
    fn test_editor_type_from_path_round_trips_all_ini_types() {
        let cases = vec![
            ("AllMap.ini", EditorType::AllMapIniEditor),
            ("Map.ini", EditorType::MapIniEditor),
            ("Extra.ini", EditorType::ExtraIniEditor),
            ("Event.ini", EditorType::EventIniEditor),
            ("Monster.ini", EditorType::MonsterIniEditor),
            ("Npc.ini", EditorType::NpcIniEditor),
            ("Wave.ini", EditorType::WaveIniEditor),
        ];
        for (filename, expected) in cases {
            let result = EditorType::from_path(PathBuf::from(filename).as_path());
            assert_eq!(result, expected, "failed for {filename}");
        }
    }

    #[test]
    fn test_editor_type_from_path_round_trips_all_ref_types() {
        let cases = vec![
            ("PartyRef.ref", EditorType::PartyRefEditor),
            ("DrawItem.ref", EditorType::DrawItemEditor),
            ("EventNpc.ref", EditorType::EventNpcRefEditor),
            ("Npc01.ref", EditorType::NpcRefEditor),
            ("Mon01.ref", EditorType::MonsterRefEditor),
            ("Ext01.ref", EditorType::ExtraRefEditor),
        ];
        for (filename, expected) in cases {
            let result = EditorType::from_path(PathBuf::from(filename).as_path());
            assert_eq!(result, expected, "failed for {filename}");
        }
    }

    #[test]
    fn test_editor_type_from_path_special_formats() {
        assert_eq!(
            EditorType::from_path(PathBuf::from("file.spr").as_path()),
            EditorType::SpriteViewer
        );
        assert_eq!(
            EditorType::from_path(PathBuf::from("file.snf").as_path()),
            EditorType::SnfEditor
        );
        assert_eq!(
            EditorType::from_path(PathBuf::from("file.dlg").as_path()),
            EditorType::DialogueScriptEditor
        );
        assert_eq!(
            EditorType::from_path(PathBuf::from("file.pgp").as_path()),
            EditorType::DialogueTextEditor
        );
        assert_eq!(
            EditorType::from_path(PathBuf::from("file.map").as_path()),
            EditorType::MapEditor
        );
        assert_eq!(
            EditorType::from_path(PathBuf::from("file.gtl").as_path()),
            EditorType::TilesetEditor
        );
        assert_eq!(
            EditorType::from_path(PathBuf::from("file.btl").as_path()),
            EditorType::TilesetEditor
        );
    }
}
