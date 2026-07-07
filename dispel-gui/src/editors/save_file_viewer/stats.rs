use iced::widget::{container, scrollable, text, Column, Row};
use iced::{Element, Fill};

use crate::editors::save_file_viewer::state::SaveFileViewerState;
use crate::message::Message;

/// 2-column grid of all 38 CharacterStats fields.
pub fn view<'a>(state: &'a SaveFileViewerState) -> Element<'a, Message> {
    let sf = match state.save_file.as_ref() {
        Some(sf) => sf,
        None => return container(text("No save file loaded")).into(),
    };

    let s = &sf.character_stats;

    scrollable(
        Column::new()
            .push(section_header("Core Attributes"))
            .push(grid_block(&[
                ("Strength", s.strength.to_string()),
                ("Agility", s.agility.to_string()),
                ("Wisdom", s.wisdom.to_string()),
                ("Constitution", s.constitution.to_string()),
                ("Morale", s.morale.to_string()),
                ("HP Current", s.hp_current.to_string()),
                ("HP Maximum", s.hp_maximum.to_string()),
                ("MP Current", s.mp_current.to_string()),
                ("MP Maximum", s.mp_maximum.to_string()),
                ("Experience", s.experience.to_string()),
                ("Level", s.level.to_string()),
                ("Gold", s.gold.to_string()),
            ]))
            .push(section_header("Combat Stats"))
            .push(grid_block(&[
                ("Offense", s.offense.to_string()),
                ("Defense", s.defense.to_string()),
                ("Dodge Rate", s.dodge_rate.to_string()),
                ("Hit Rate", s.hit_rate.to_string()),
                ("Magic Power", s.magic_power.to_string()),
                ("Attack Modifier", s.attack_modifier.to_string()),
            ]))
            .push(section_header("Skills"))
            .push(grid_block(&[
                ("Thievery", s.thievery.to_string()),
                ("Lockpicking", s.lockpicking.to_string()),
                ("Haggling", s.haggling.to_string()),
                ("Perception", s.perception.to_string()),
                ("Traps", s.traps.to_string()),
            ]))
            .push(section_header("Weapon Skills"))
            .push(grid_block(&[
                ("Swords Level", s.swords_level.to_string()),
                ("Swords Kills", s.swords_kills.to_string()),
                ("Axes Level", s.axes_level.to_string()),
                ("Axes Kills", s.axes_kills.to_string()),
                ("Archery Level", s.archery_level.to_string()),
                ("Archery Kills", s.archery_kills.to_string()),
                ("Polearm Level", s.polearm_level.to_string()),
                ("Polearm Kills", s.polearm_kills.to_string()),
                ("Magic Level", s.magic_level.to_string()),
                ("Magic Kills", s.magic_kills.to_string()),
                ("Holy Magic Level", s.holy_magic_level.to_string()),
                ("Holy Magic Kills", s.holy_magic_kills.to_string()),
                ("Dark Magic Level", s.dark_magic_level.to_string()),
                ("Dark Magic Kills", s.dark_magic_kills.to_string()),
            ]))
            .spacing(8)
            .padding(16),
    )
    .into()
}

fn section_header(label: &str) -> Element<'static, Message> {
    container(text(label.to_string()).size(16))
        .padding([8, 0])
        .width(Fill)
        .into()
}

fn grid_block(pairs: &[(&str, String)]) -> Element<'static, Message> {
    let mut col_a: Vec<Element<'static, Message>> = Vec::new();
    let mut col_b: Vec<Element<'static, Message>> = Vec::new();
    for (i, (label, value)) in pairs.iter().enumerate() {
        let entry = cell_row(label, value);
        if i % 2 == 0 {
            col_a.push(entry);
        } else {
            col_b.push(entry);
        }
    }
    Row::new()
        .push(Column::new().spacing(4).width(Fill).extend(col_a))
        .push(Column::new().spacing(4).width(Fill).extend(col_b))
        .spacing(32)
        .into()
}

fn cell_row(label: &str, value: &str) -> Element<'static, Message> {
    Row::new()
        .push(text(label.to_string()).width(150))
        .push(text(value.to_string()))
        .spacing(8)
        .into()
}
