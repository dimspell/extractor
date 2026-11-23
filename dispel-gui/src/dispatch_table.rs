//! Editor message dispatch.
//!
//! The update side is generated once from [`define_update_dispatch!`]; the
//! view / save / load / spreadsheet dispatches are derived from the
//! authoritative editor catalog in `crate::editor_catalog` and re-exported
//! here for backwards compatibility.
//!
//! One editor is handled specially outside the update table:
//! - `EventScrEditor` — view wraps result in `.map(EditorMessage::EventScr)`

use crate::app::App;
use crate::message::Message;
use crate::message::editor::EditorMessage;
use iced::Task;

// Descriptor-driven dispatch lives in the catalog; keep the historical paths
// used by callers (`dispatch_table::dispatch_view`, `::load_catalog_task`, …).
pub use crate::editor_catalog::{
    catalog_load_task as load_catalog_task, dispatch_view, save_task_for_editor,
    spreadsheet_nav_msg,
};

/// Generate update dispatch function (message routing).
/// EventScr is included here (standard) but NOT in view dispatch (wrapping needed).
macro_rules! define_update_dispatch {
    ($(($msg:ident, $mod:ident)),+ $(,)?) => {
        /// Route EditorMessage variants to the correct editor's handle().
        pub fn dispatch(message: EditorMessage, app: &mut App) -> Task<Message> {
            match message {
                $(EditorMessage::$msg(msg) => crate::editors::$mod::handle(msg, app),)+
                // Save messages that carry their own tab id bypass the
                // active-tab lookup in the editor handlers.
                EditorMessage::HexEditorSave(tab_id) => {
                    crate::editors::hex_wrapper::handle_for_tab(
                        tab_id,
                        hexedit::HexEditorMessage::SaveIntoRecording,
                        app,
                    )
                }
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
    (SaveFileViewer, save_file_viewer),
    (SaveIfo, save_ifo),
    (FogData, fog_data),
    (ChestSpawn, chest_spawn),
    (MonsterSpawn, monster_spawn),
    (MapInfoMulti, map_info_multi),
    (TileGrid, tile_grid),
    (TileGridCanvas, tile_grid_canvas),
}
