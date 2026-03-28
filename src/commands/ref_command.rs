use super::super::references::{
    all_map_ini, chdata_db, dialog, draw_item, edit_item_db, event_ini, event_item_db,
    event_npc_ref, extra_ini, extra_ref, heal_item_db, magic_db, map_ini, message_scr,
    misc_item_db, monster_db, monster_ini, monster_ref, npc_ini, npc_ref, party_ini_db,
    party_level_db, party_pgp, party_ref, quest_scr, store_db, wave_ini, weapons_db,
};
use super::Command;
use std::error::Error;
use std::path::Path;

/// Reference command implementation
pub struct RefCommand {
    pub subcommand: RefSubcommand,
}

pub enum RefSubcommand {
    AllMaps { input: String },
    Map { input: String },
    Extra { input: String },
    Event { input: String },
    Monster { input: String },
    Npc { input: String },
    Wave { input: String },
    PartyRef { input: String },
    DrawItem { input: String },
    PartyPgp { input: String },
    PartyDialog { input: String },
    Dialog { input: String },
    Weapons { input: String },
    MultiMagic { input: String },
    Store { input: String },
    NpcRef { input: String },
    MonsterRef { input: String },
    Monsters { input: String },
    MiscItem { input: String },
    HealItems { input: String },
    ExtraRef { input: String },
    EventItems { input: String },
    EditItems { input: String },
    PartyLevel { input: String },
    PartyIni { input: String },
    EventNpcRef { input: String },
    Magic { input: String },
    Quest { input: String },
    Message { input: String },
    ChData { input: String },
}

impl Command for RefCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        match &self.subcommand {
            RefSubcommand::AllMaps { input } => {
                let data = all_map_ini::read_all_map_ini(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::Map { input } => {
                let data =
                    map_ini::read_map_ini(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::Extra { input } => {
                let data = extra_ini::read_extra_ini(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::Event { input } => {
                let data = event_ini::read_event_ini(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::Monster { input } => {
                let data = monster_ini::read_monster_ini(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::Npc { input } => {
                let data =
                    npc_ini::read_npc_ini(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::Wave { input } => {
                let data =
                    wave_ini::read_wave_ini(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::DrawItem { input } => {
                let data = draw_item::read_draw_items(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::Dialog { input } => {
                let data =
                    dialog::read_dialogs(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::PartyRef { input } => {
                let data = party_ref::read_part_refs(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::PartyPgp { input } => {
                let data = party_pgp::read_party_pgps(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::PartyDialog { input } => {
                let data =
                    dialog::read_dialogs(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::Weapons { input } => {
                let data = weapons_db::read_weapons_db(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::MultiMagic { input } => {
                let data =
                    super::super::references::references::read_mutli_magic_db(&Path::new(input))
                        .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::Store { input } => {
                let data =
                    store_db::read_store_db(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::EventNpcRef { input } => {
                let data = event_npc_ref::read_event_npc_ref(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::NpcRef { input } => {
                let data =
                    npc_ref::read_npc_ref(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::Monsters { input } => {
                let data = monster_db::read_monster_db(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::MonsterRef { input } => {
                let data = monster_ref::read_monster_ref(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::MiscItem { input } => {
                let data = misc_item_db::read_misc_item_db(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::HealItems { input } => {
                let data = heal_item_db::read_heal_item_db(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::ExtraRef { input } => {
                let data = extra_ref::read_extra_ref(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::EventItems { input } => {
                let data = event_item_db::read_event_item_db(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::EditItems { input } => {
                let data = edit_item_db::read_edit_item_db(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::PartyLevel { input } => {
                let data = party_level_db::read_party_level_db(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::PartyIni { input } => {
                let data = party_ini_db::read_party_ini_db(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::Magic { input } => {
                let data =
                    magic_db::read_magic_db(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::Quest { input } => {
                let data =
                    quest_scr::read_quests(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::Message { input } => {
                let data = message_scr::read_messages(&Path::new(input))
                    .expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
            RefSubcommand::ChData { input } => {
                let data =
                    chdata_db::read_chdata(&Path::new(input)).expect("ERROR: could not read file");
                println!(
                    "{}",
                    serde_json::to_string(&data).expect("ERROR: could not encode JSON")
                );
            }
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "ref"
    }

    fn description(&self) -> &'static str {
        "Convert game DB/INI/REF files to JSON"
    }
}
