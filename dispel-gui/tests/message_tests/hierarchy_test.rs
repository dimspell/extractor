// Test the generated message hierarchy
use dispel_gui::message::*;

#[test]
fn test_message_hierarchy_construction() {
    // Test that we can construct messages using the new hierarchy
    let weapon_msg = Message::weapon(WeaponEditorMessage::ScanWeapons);
    let monster_msg = Message::Editor(EditorMessage::Monster(MonsterEditorMessage::SelectMonster(
        0,
    )));
    let workspace_msg = Message::Workspace(WorkspaceMessage::ToggleWorkspaceMode);

    // Verify the messages can be matched
    match weapon_msg {
        Message::weapon(WeaponEditorMessage::ScanWeapons) => {}
        _ => panic!("Message matching failed"),
    }
}

#[test]
fn test_all_editor_messages() {
    // Test that all editor message types can be constructed
    let editors = vec![
        EditorMessage::Weapon(WeaponEditorMessage::ScanWeapons),
        EditorMessage::Monster(MonsterEditorMessage::SelectMonster(0)),
        EditorMessage::HealItem(HealItemEditorMessage::ScanItems),
        EditorMessage::MiscItem(MiscItemEditorMessage::ScanItems),
        EditorMessage::EditItem(EditItemEditorMessage::ScanItems),
        EditorMessage::EventItem(EventItemEditorMessage::ScanItems),
        EditorMessage::NpcIni(NpcIniEditorMessage::ScanNpcs),
        EditorMessage::Magic(MagicEditorMessage::ScanSpells),
        EditorMessage::Store(StoreEditorMessage::ScanStores),
        EditorMessage::PartyRef(PartyRefEditorMessage::ScanParty),
        EditorMessage::PartyIni(PartyIniEditorMessage::ScanNpcs),
        EditorMessage::SpriteBrowser(SpriteBrowserEditorMessage::Scan),
        EditorMessage::MonsterRef(MonsterRefEditorMessage::ScanFiles),
        EditorMessage::AllMapIni(AllMapIniEditorMessage::SelectMap(0)),
        EditorMessage::Dialog(DialogEditorMessage::ScanFiles),
        EditorMessage::DialogueText(DialogueTextEditorMessage::ScanFiles),
        EditorMessage::DrawItem(DrawItemEditorMessage::SelectItem(0)),
        EditorMessage::EventIni(EventIniEditorMessage::SelectEvent(0)),
        EditorMessage::EventNpcRef(EventNpcRefEditorMessage::SelectNpc(0)),
        EditorMessage::ExtraIni(ExtraIniEditorMessage::SelectExtra(0)),
        EditorMessage::ExtraRef(ExtraRefEditorMessage::SelectItem(0)),
        EditorMessage::MapIni(MapIniEditorMessage::SelectMap(0)),
        EditorMessage::MessageScr(MessageScrEditorMessage::SelectMessage(0)),
        EditorMessage::NpcRef(NpcRefEditorMessage::ScanFiles),
        EditorMessage::PartyLevelDb(PartyLevelDbEditorMessage::SelectRecord(0)),
        EditorMessage::QuestScr(QuestScrEditorMessage::SelectQuest(0)),
        EditorMessage::WaveIni(WaveIniEditorMessage::SelectWave(0)),
        EditorMessage::ChData(ChDataEditorMessage::SelectData(0)),
    ];

    // All should be constructable without panic
    for editor_msg in editors {
        let _ = Message::Editor(editor_msg);
    }
}

#[test]
fn test_domain_messages() {
    // Test domain-specific messages
    let workspace_msg = Message::Workspace(WorkspaceMessage::ToggleWorkspaceMode);
    let filetree_msg = Message::FileTree(FileTreeMessage::Browse);
    let viewer_msg = Message::Viewer(ViewerMessage::DbPathChanged("test".to_string()));
    let system_msg = Message::System(SystemMessage::CloseRequested);

    // Verify they can be matched
    match workspace_msg {
        Message::Workspace(WorkspaceMessage::ToggleWorkspaceMode) => {}
        _ => panic!("Workspace message matching failed"),
    }
}
