use iced::Element;
use iced::widget::{Column, container, scrollable, text};

use crate::editors::save_file_viewer::helpers::{label_row, section_header};
use crate::editors::save_file_viewer::state::SaveFileViewerState;
use crate::message::Message;

/// Compact overview card showing character summary.
pub fn view<'a>(state: &'a SaveFileViewerState) -> Element<'a, Message> {
    let sf = match state.save_file.as_ref() {
        Some(sf) => sf,
        None => return container(text("No save file loaded")).into(),
    };

    scrollable(
        Column::new()
            .push(section_header("Character Overview"))
            .push(label_row(
                "Player",
                sf.character_identity.player_name.clone(),
            ))
            .push(label_row(
                "Class",
                format!(
                    "{} (ID: {})",
                    sf.character_identity.player_class_name, sf.character_identity.player_class_id
                ),
            ))
            .push(label_row("Level", sf.character_stats.level.to_string()))
            .push(label_row("Gold", sf.character_stats.gold.to_string()))
            .push(label_row(
                "HP",
                format!(
                    "{}/{}",
                    sf.character_stats.hp_current, sf.character_stats.hp_maximum
                ),
            ))
            .push(label_row(
                "MP",
                format!(
                    "{}/{}",
                    sf.character_stats.mp_current, sf.character_stats.mp_maximum
                ),
            ))
            .push(section_header("Sprite Paths"))
            .extend(
                sf.sprite_paths
                    .iter()
                    .enumerate()
                    .map(|(i, path)| label_row(format!("Sprite {}", i + 1), path.clone())),
            )
            .push(section_header("Save Metadata"))
            .push(label_row(
                "Game Version",
                sf.post_maps.game_version.to_string(),
            ))
            .push(label_row(
                "AllMap.ini ID",
                sf.post_maps.all_map_ini_id.to_string(),
            ))
            .push(label_row(
                "Ref/Map.ini ID",
                sf.post_maps.ref_map_ini_id.to_string(),
            ))
            .push(label_row(
                "Unknowns A",
                format!(
                    "{}, {}, {}",
                    sf.post_maps.unknowns_a[0],
                    sf.post_maps.unknowns_a[1],
                    sf.post_maps.unknowns_a[2]
                ),
            ))
            .push(label_row(
                "Monster Block Size",
                sf.post_maps.monster_block_size.to_string(),
            ))
            .push(label_row(
                "NPC Block Size",
                sf.post_maps.npc_block_size.to_string(),
            ))
            .push(label_row(
                "Extra Object Block Size",
                sf.post_maps.extra_object_block_size.to_string(),
            ))
            .push(label_row(
                "Visited Maps",
                sf.post_maps.number_of_visited_maps.to_string(),
            ))
            .push(label_row(
                "Position X (tile)",
                sf.character_position_x.to_string(),
            ))
            .push(label_row(
                "Position Y (tile)",
                sf.character_position_y.to_string(),
            ))
            .push(label_row(
                "Selected Spell ID",
                sf.character_stats_header.selected_spell_id.to_string(),
            ))
            .push(label_row(
                "Map IDs",
                if sf.post_maps.map_ids.is_empty() {
                    "(none)".to_string()
                } else {
                    let mut s = sf
                        .post_maps
                        .map_ids
                        .iter()
                        .take(20)
                        .map(|id| id.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    if sf.post_maps.map_ids.len() > 20 {
                        s.push_str(&format!(" … ({} total)", sf.post_maps.map_ids.len()));
                    }
                    s
                },
            ))
            .spacing(4)
            .padding(16),
    )
    .into()
}
