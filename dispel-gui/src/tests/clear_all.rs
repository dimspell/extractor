#[cfg(test)]
mod clear_all_tests {
    use crate::app::App;
    use crate::workspace::Workspace;

    #[test]
    fn clear_all_resets_monster_ini_editor() {
        let mut app = App::test_new(Workspace::new());
        app.state.editors.monster_ini_editor.state.catalog = Some(vec![]);
        assert!(app.state.editors.monster_ini_editor.state.catalog.is_some());

        app.state.editors.clear_all();

        assert!(
            app.state.editors.monster_ini_editor.state.catalog.is_none(),
            "monster_ini_editor should be reset after clear_all()"
        );
    }

    #[test]
    fn clear_all_resets_viewer() {
        let mut app = App::test_new(Workspace::new());
        app.state.editors.viewer.db_path = "test.db".into();
        assert_eq!(app.state.editors.viewer.db_path, "test.db");

        app.state.editors.clear_all();

        assert_eq!(
            app.state.editors.viewer.db_path, "database.sqlite",
            "viewer should reset to default db_path after clear_all()"
        );
    }

    #[test]
    fn clear_all_resets_party_level_db_editor() {
        let mut app = App::test_new(Workspace::new());
        app.state.editors.party_level_db_editor.catalog = Some(vec![]);
        assert!(app.state.editors.party_level_db_editor.catalog.is_some());

        app.state.editors.clear_all();

        assert!(
            app.state.editors.party_level_db_editor.catalog.is_none(),
            "party_level_db_editor should be reset after clear_all()"
        );
    }

    #[test]
    fn clear_all_resets_party_level_db_level_editor() {
        let mut app = App::test_new(Workspace::new());
        app.state.editors.party_level_db_level_editor.state.catalog = Some(vec![]);
        assert!(app
            .state
            .editors
            .party_level_db_level_editor
            .state
            .catalog
            .is_some());

        app.state.editors.clear_all();

        assert!(
            app.state
                .editors
                .party_level_db_level_editor
                .state
                .catalog
                .is_none(),
            "party_level_db_level_editor should be reset after clear_all()"
        );
    }

    #[test]
    fn clear_all_does_not_panic_on_fresh_registry() {
        let mut app = App::test_new(Workspace::new());
        app.state.editors.clear_all(); // no panic
    }
}
