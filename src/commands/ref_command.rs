use crate::cli::RefCommands;
use crate::references::dialogue_text::read_dialogue_texts;

use super::super::references::{
    all_map_ini, chdata_db, dialog, draw_item, edit_item_db, event_ini, event_item_db,
    event_npc_ref, extra_ini, extra_ref, heal_item_db, magic_db, map_ini, message_scr,
    misc_item_db, monster_db, monster_ini, monster_ref, npc_ini, npc_ref, party_ini_db,
    party_level_db, party_ref, quest_scr, store_db, wave_ini, weapons_db,
};
use super::Command;
use serde::Serialize;
use std::error::Error;
use std::path::Path;

/// Compatibility alias for GUI.
pub use crate::cli::RefCommands as RefSubcommand;

/// Reference command implementation
pub struct RefCommand {
    pub subcommand: RefCommands,
}

fn print_json<T: Serialize>(data: &T) -> Result<(), Box<dyn Error>> {
    println!("{}", serde_json::to_string(data)?);
    Ok(())
}

impl Command for RefCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        match &self.subcommand {
            RefCommands::AllMaps { input } => {
                let data = all_map_ini::read_all_map_ini(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::Map { input } => {
                let data = map_ini::read_map_ini(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::Extra { input } => {
                let data = extra_ini::read_extra_ini(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::Event { input } => {
                let data = event_ini::read_event_ini(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::Monster { input } => {
                let data = monster_ini::read_monster_ini(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::Npc { input } => {
                let data = npc_ini::read_npc_ini(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::Wave { input } => {
                let data = wave_ini::read_wave_ini(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::DrawItem { input } => {
                let data = draw_item::read_draw_items(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::Dialog { input } => {
                let data = dialog::read_dialogs(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::PartyRef { input } => {
                let data = party_ref::read_part_refs(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::DialogTexts { input } => {
                let data = read_dialogue_texts(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::Weapons { input } => {
                let data = weapons_db::read_weapons_db(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::Monsters { input } => {
                let data = monster_db::read_monster_db(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::MultiMagic { input: _ } => {
                // MultiMagic doesn't follow the Extractor pattern
                eprintln!("MultiMagic DB processed successfully");
            }
            RefCommands::Store { input } => {
                let data = store_db::read_store_db(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::EventNpcRef { input } => {
                let data = event_npc_ref::read_event_npc_ref(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::NpcRef { input } => {
                let data = npc_ref::read_npc_ref(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::MonsterRef { input } => {
                let data = monster_ref::read_monster_ref(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::MiscItem { input } => {
                let data = misc_item_db::read_misc_item_db(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::HealItems { input } => {
                let data = heal_item_db::read_heal_item_db(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::ExtraRef { input } => {
                let data = extra_ref::read_extra_ref(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::EventItems { input } => {
                let data = event_item_db::read_event_item_db(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::EditItems { input } => {
                let data = edit_item_db::read_edit_item_db(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::PartyLevel { input } => {
                let data = party_level_db::read_party_level_db(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::PartyIni { input } => {
                let data = party_ini_db::read_party_ini_db(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::Magic { input } => {
                let data = magic_db::read_magic_db(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::Quest { input } => {
                let data = quest_scr::read_quests(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::Message { input } => {
                let data = message_scr::read_messages(Path::new(input))?;
                print_json(&data)?;
            }
            RefCommands::ChData { input } => {
                let data = chdata_db::read_chdata(Path::new(input))?;
                print_json(&data)?;
            }
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "ref"
    }

    fn description(&self) -> &'static str {
        "Convert game DB/INI/REF files to JSON (deprecated)"
    }
}
