//! Generated editor dispatch table.
//!
//! Collapses the 37-arm message dispatch and the 35-arm view dispatch
//! into a single macro declaration. Two editors are handled specially
//! outside this table:
//! - `DbViewer` — no EditorMessage variant; view is `App::view_db_viewer()`
//! - `EventScrEditor` — view wraps result in `.map(EditorMessage::EventScr)`

use crate::app::App;
use crate::message::editor::EditorMessage;
use crate::message::Message;
use crate::workspace::EditorType;
use iced::Task;

/// Generate update dispatch function (message routing).
/// EventScr is included here (standard) but NOT in view dispatch (wrapping needed).
macro_rules! define_update_dispatch {
    ($(($msg:ident, $mod:ident)),+ $(,)?) => {
        /// Route EditorMessage variants to the correct editor's handle().
        pub fn dispatch(message: EditorMessage, app: &mut App) -> Task<Message> {
            match message {
                $(EditorMessage::$msg(msg) => crate::editors::$mod::handle(msg, app),)+
            }
        }
    };
}

/// Generate view dispatch function.
/// Excludes EventScrEditor (wrapping handled by caller) and DbViewer (no EditorType).
macro_rules! define_view_dispatch {
    ($(($et:ident, $mod:ident)),+ $(,)?) => {
        /// Route EditorType variants to the correct editor's view().
        /// Returns None for types not in this table (handled specially by the caller).
        pub fn dispatch_view<'a>(
            editor_type: Option<EditorType>,
            app: &'a App,
        ) -> Option<iced::Element<'a, Message>> {
            let et = editor_type?;
            match et {
                $(EditorType::$et => Some(crate::editors::$mod::view(app)),)+
                _ => None,
            }
        }
    };
}

define_update_dispatch! {
    (Weapon, weapon),
    (Monster, monster),
    (HealItem, heal_item),
    (MiscItem, misc_item),
    (EditItem, edit_item),
    (EventItem, event_item),
    (NpcIni, npc_ini),
    (MonsterIni, monster_ini),
    (Magic, magic),
    (Store, store),
    (PartyRef, party_ref),
    (PartyIni, party_ini),
    (SpriteViewer, sprite_editor),
    (MonsterRef, monster_ref),
    (AllMapIni, all_map_ini),
    (DialogueScript, dialogue_script),
    (DialogueParagraph, dialogue_paragraph),
    (DrawItem, draw_item),
    (EventIni, event_ini),
    (EventNpcRef, event_npc_ref),
    (ExtraIni, extra_ini),
    (ExtraRef, extra_ref),
    (MapIni, map_ini),
    (MessageScr, message_scr),
    (NpcRef, npc_ref),
    (PartyLevelDb, party_level_db),
    (QuestScr, quest_scr),
    (EventScr, event_scr),
    (WaveIni, wave_ini),
    (ChData, chdata),
    (MapEditor, map_editor),
    (Tileset, tileset),
    (Snf, snf_editor),
    (ModPackager, mod_packager),
    (Localization, localization_manager),
    (HexEditor, hex_wrapper),
}

define_view_dispatch! {
    (WeaponEditor, weapon),
    (MonsterEditor, monster),
    (HealItemEditor, heal_item),
    (MiscItemEditor, misc_item),
    (EditItemEditor, edit_item),
    (EventItemEditor, event_item),
    (NpcIniEditor, npc_ini),
    (MonsterIniEditor, monster_ini),
    (MagicEditor, magic),
    (StoreEditor, store),
    (PartyRefEditor, party_ref),
    (PartyIniEditor, party_ini),
    (SpriteViewer, sprite_editor),
    (MonsterRefEditor, monster_ref),
    (AllMapIniEditor, all_map_ini),
    (DialogueScriptEditor, dialogue_script),
    (DialogueTextEditor, dialogue_paragraph),
    (DrawItemEditor, draw_item),
    (EventIniEditor, event_ini),
    (EventNpcRefEditor, event_npc_ref),
    (ExtraIniEditor, extra_ini),
    (ExtraRefEditor, extra_ref),
    (MapIniEditor, map_ini),
    (MessageScrEditor, message_scr),
    (NpcRefEditor, npc_ref),
    (PartyLevelDbEditor, party_level_db),
    (QuestScrEditor, quest_scr),
    // EventScrEditor intentionally omitted -- handled by caller with wrapping
    (WaveIniEditor, wave_ini),
    (ChDataEditor, chdata),
    (MapEditor, map_editor),
    (TilesetEditor, tileset),
    (SnfEditor, snf_editor),
    (ModPackager, mod_packager),
    (LocalizationManager, localization_manager),
    (HexEditor, hex_wrapper),
}

/// Return a `LoadCatalog` task for editors that load from the configured game
/// path. Returns `None` for editors that are opened by explicit file path
/// (dialogue, tileset, map, ref files) or that have no load-on-start behaviour.
pub fn load_catalog_task(et: EditorType) -> Option<iced::Task<Message>> {
    use crate::components::standard::message::StandardEditorMessage;
    use crate::message::MessageExt;

    macro_rules! load {
        ($wrap:expr) => {
            Some(iced::Task::done($wrap(StandardEditorMessage::LoadCatalog)))
        };
    }

    use EditorType::*;
    match et {
        WeaponEditor => load!(Message::weapon),
        HealItemEditor => load!(Message::heal_item),
        MiscItemEditor => load!(Message::misc_item),
        EditItemEditor => load!(Message::edit_item),
        EventItemEditor => load!(Message::event_item),
        MonsterEditor => load!(Message::monster),
        MonsterIniEditor => load!(Message::monster_ini),
        NpcIniEditor => load!(Message::npc_ini),
        MagicEditor => load!(Message::magic),
        PartyRefEditor => load!(Message::party_ref),
        PartyIniEditor => load!(Message::party_ini),
        AllMapIniEditor => load!(Message::all_map_ini),
        MapIniEditor => load!(Message::map_ini),
        ExtraIniEditor => load!(Message::extra_ini),
        EventIniEditor => load!(Message::event_ini),
        WaveIniEditor => Some(iced::Task::done(Message::wave_ini(
            crate::editors::wave_ini::WaveIniEditorMessage::LoadCatalog,
        ))),
        DrawItemEditor => load!(Message::draw_item),
        EventNpcRefEditor => load!(Message::event_npc_ref),
        QuestScrEditor => load!(Message::quest_scr),
        MessageScrEditor => load!(Message::message_scr),
        ChDataEditor => load!(Message::chdata),
        StoreEditor => Some(iced::Task::done(Message::store(
            crate::editors::store::StoreEditorMessage::LoadCatalog,
        ))),
        PartyLevelDbEditor => Some(iced::Task::done(Message::party_level_db(
            crate::editors::party_level_db::PartyLevelDbEditorMessage::LoadCatalog,
        ))),
        _ => None,
    }
}

/// Map `(EditorType, SpreadsheetMessage)` to the correct `Message` variant.
/// Returns `None` for editor types that have no spreadsheet (map editor, sprite
/// viewer, etc.) so callers can use this as a capability check.
pub fn spreadsheet_nav_msg(
    et: EditorType,
    sm: crate::view::editor::SpreadsheetMessage,
) -> Option<Message> {
    use crate::editors::*;
    use crate::message::MessageExt as _;
    use EditorType::*;
    Some(match et {
        WeaponEditor => Message::weapon(weapon::WeaponEditorMessage::Spreadsheet(sm)),
        MonsterEditor => Message::monster(monster::MonsterEditorMessage::Spreadsheet(sm)),
        MonsterIniEditor => {
            Message::monster_ini(monster_ini::MonsterIniEditorMessage::Spreadsheet(sm))
        }
        HealItemEditor => Message::heal_item(heal_item::HealItemEditorMessage::Spreadsheet(sm)),
        MiscItemEditor => Message::misc_item(misc_item::MiscItemEditorMessage::Spreadsheet(sm)),
        EditItemEditor => Message::edit_item(edit_item::EditItemEditorMessage::Spreadsheet(sm)),
        EventItemEditor => Message::event_item(event_item::EventItemEditorMessage::Spreadsheet(sm)),
        MagicEditor => Message::magic(magic::MagicEditorMessage::Spreadsheet(sm)),
        StoreEditor => return None, // Store editor has a custom layout, no generic spreadsheet
        NpcIniEditor => Message::npc_ini(npc_ini::NpcIniEditorMessage::Spreadsheet(sm)),
        NpcRefEditor => Message::npc_ref(npc_ref::NpcRefEditorMessage::Spreadsheet(sm)),
        MonsterRefEditor => {
            Message::monster_ref(monster_ref::MonsterRefEditorMessage::Spreadsheet(sm))
        }
        PartyRefEditor => Message::party_ref(party_ref::PartyRefEditorMessage::Spreadsheet(sm)),
        PartyIniEditor => Message::party_ini(party_ini::PartyIniEditorMessage::Spreadsheet(sm)),
        AllMapIniEditor => {
            Message::all_map_ini(all_map_ini::AllMapIniEditorMessage::Spreadsheet(sm))
        }
        MapIniEditor => Message::map_ini(map_ini::MapIniEditorMessage::Spreadsheet(sm)),
        ExtraIniEditor => Message::extra_ini(extra_ini::ExtraIniEditorMessage::Spreadsheet(sm)),
        ExtraRefEditor => Message::extra_ref(extra_ref::ExtraRefEditorMessage::Spreadsheet(sm)),
        EventIniEditor => Message::event_ini(event_ini::EventIniEditorMessage::Spreadsheet(sm)),
        EventNpcRefEditor => {
            Message::event_npc_ref(event_npc_ref::EventNpcRefEditorMessage::Spreadsheet(sm))
        }
        WaveIniEditor => Message::wave_ini(wave_ini::WaveIniEditorMessage::Spreadsheet(sm)),
        DrawItemEditor => Message::draw_item(draw_item::DrawItemEditorMessage::Spreadsheet(sm)),
        MessageScrEditor => {
            Message::message_scr(message_scr::MessageScrEditorMessage::Spreadsheet(sm))
        }
        QuestScrEditor => Message::quest_scr(quest_scr::QuestScrEditorMessage::Spreadsheet(sm)),
        DialogueScriptEditor => Message::dialogue_script(
            dialogue_script::DialogueScriptEditorMessage::Spreadsheet(sm),
        ),
        DialogueTextEditor => Message::dialogue_paragraph(
            dialogue_paragraph::DialogueParagraphEditorMessage::Spreadsheet(sm),
        ),
        ChDataEditor => Message::chdata(chdata::ChDataEditorMessage::Spreadsheet(sm)),
        PartyLevelDbEditor => {
            Message::party_level_db(party_level_db::PartyLevelDbEditorMessage::Spreadsheet(sm))
        }
        _ => return None,
    })
}
