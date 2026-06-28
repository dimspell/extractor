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
    (Chest, chest),
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
    (ChestEditor, chest),
    (MapEditor, map_editor),
    (TilesetEditor, tileset),
    (SnfEditor, snf_editor),
    (ModPackager, mod_packager),
    (LocalizationManager, localization_manager),
    (HexEditor, hex_wrapper),
}
