use crate::editors::save_file_viewer::helpers::{label_row, section_header};
use crate::editors::save_file_viewer::state::SaveFileViewerState;
use crate::message::Message;
use dispel_core::references::save_file::PartyMember;
use iced::Element;
use iced::widget::{Column, container, scrollable, text};

/// Player party member section
pub fn view<'a>(state: &'a SaveFileViewerState) -> Element<'a, Message> {
    let sf = match state.save_file.as_ref() {
        Some(sf) => sf,
        None => return container(text("No save file loaded")).into(),
    };

    let identity = &sf.character_identity;
    let party_members = &sf.character_identity.party_members;

    let character_header = &identity.character_data_header;

    scrollable(
        Column::new()
            .push(section_header("Character Data Header"))
            .push(label_row(
                "unknown_a",
                character_header.unknown_a.to_string(),
            ))
            .push(label_row(
                "unknown_b",
                character_header.unknown_b.to_string(),
            ))
            .push(label_row(
                "unknown_c",
                character_header.unknown_c.to_string(),
            ))
            .push(label_row(
                "unknown_d",
                character_header.unknown_d.to_string(),
            ))
            .push(label_row(
                "unknown_e",
                character_header.unknown_e.to_string(),
            ))
            .push(label_row(
                "unknown_f",
                character_header.unknown_f.to_string(),
            ))
            .push(label_row(
                "Equipment Slots",
                identity.equipped_equipment.len().to_string(),
            ))
            .push(label_row(
                "Belt Potions",
                identity.belt_potions.len().to_string(),
            ))
            .push(label_row(
                "Inventory Placements",
                identity.inventory_placement.len().to_string(),
            ))
            .push(label_row(
                "Learned Spells",
                identity.learned_spells.spells.len().to_string(),
            ))
            .push(section_header("Learned Spells"))
            .push(identity.learned_spells.spells.iter().enumerate().fold(
                Column::new().spacing(4),
                |col, (i, flag)| {
                    col.push(label_row(
                        format!("Spell {:02}", i + 1),
                        if *flag != 0 { "learned" } else { "not learned" },
                    ))
                },
            ))
            .push(section_header("Player Identity"))
            .push(label_row(
                "Party Members Count",
                identity.party_members_count.to_string(),
            ))
            .push(Column::new().spacing(4).push(match party_members.first() {
                Some(member) => party_member_block(member),
                None => text("No party member in slot 1").into(),
            }))
            .push(Column::new().spacing(4).push(match party_members.get(1) {
                Some(member) => party_member_block(member),
                None => text("No party member in slot 2").into(),
            }))
            .spacing(8)
            .padding(16),
    )
    .into()
}

fn party_member_block(member: &PartyMember) -> Element<'static, Message> {
    container(
        Column::new()
            .push(text(member.name.to_string()).size(16))
            .push(label_row("Class ID", member.class_id.to_string()))
            .push(label_row("Level", member.level.to_string()))
            .push(label_row(
                "HP",
                format!(
                    "{} / {}",
                    member.current_health_points, member.maximum_health_points
                ),
            ))
            .push(label_row(
                "MP",
                format!(
                    "{} / {}",
                    member.current_mana_points, member.maximum_mana_points
                ),
            ))
            .push(label_row("Strength", member.strength.to_string()))
            .push(label_row("Constitution", member.constitution.to_string()))
            .push(label_row("Wisdom", member.wisdom.to_string()))
            .push(label_row("Agility", member.agility.to_string()))
            .push(label_row("Attack", member.attack.to_string()))
            .push(label_row(
                "Spells",
                format!(
                    "{}, {}, {}",
                    member.magic_spell_id_1, member.magic_spell_id_2, member.magic_spell_id_3
                ),
            ))
            .push(label_row("XP", member.experience_points.to_string()))
            .push(label_row(
                "Tactical Action Chance",
                format!("{}%", member.tactical_action_chance),
            ))
            .push(label_row(
                "Pathfinding Mode",
                member.pathfinding_mode.to_string(),
            ))
            .spacing(3),
    )
    .padding(8)
    .into()
}
