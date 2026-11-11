#[cfg(all(test, feature = "iced_test"))]
mod view_dispatch_tests {
    use crate::tests::app_with_tab;
    use crate::workspace::EditorType::*;
    use iced_test::simulator;

    #[test]
    fn test_all_editor_types_render_without_panic() {
        let types = vec![
            WeaponEditor,
            MonsterEditor,
            MonsterIniEditor,
            HealItemEditor,
            MiscItemEditor,
            EditItemEditor,
            EventItemEditor,
            MagicEditor,
            StoreEditor,
            ChDataEditor,
            PartyLevelDbEditor,
            DialogueScriptEditor,
            DialogueTextEditor,
            DrawItemEditor,
            EventIniEditor,
            EventNpcRefEditor,
            ExtraIniEditor,
            ExtraRefEditor,
            MapIniEditor,
            MessageScrEditor,
            MonsterRefEditor,
            NpcIniEditor,
            NpcRefEditor,
            PartyRefEditor,
            PartyIniEditor,
            QuestScrEditor,
            EventScrEditor,
            WaveIniEditor,
            AllMapIniEditor,
            SpriteViewer,
            SnfEditor,
            DbViewer,
            TilesetEditor,
            MapEditor,
            ModPackager,
            LocalizationManager,
            HexEditor,
            FogDataEditor,
        ];

        for et in types {
            let app = app_with_tab(et);
            let view = app.view();
            // iced_test runs widget layout, catching layout panics:
            let _ui = simulator(view);
            // If we get here, no panic.
        }
    }
}
