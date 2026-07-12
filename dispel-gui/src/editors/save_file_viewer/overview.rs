use iced::widget::{container, scrollable, text, Column};
use iced::{Element, Fill};

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
            .push(label_row("Player", sf.character_identity.player_name.clone()))
            .push(label_row(
                "Class",
                format!(
                    "{} (ID: {})",
                    sf.character_identity.player_class_name,
                    sf.character_identity.player_class_id
                ),
            ))
            .push(label_row("Level", sf.character_stats.level.to_string()))
            .push(label_row("Gold", sf.character_stats.gold.to_string()))
            .push(label_row(
                "HP",
                format!("{}/{}", sf.character_stats.hp_current, sf.character_stats.hp_maximum),
            ))
            .push(label_row(
                "MP",
                format!("{}/{}", sf.character_stats.mp_current, sf.character_stats.mp_maximum),
            ))
            .push(section_header("Sprite Paths"))
            .extend(sf.sprite_paths.iter().enumerate().map(|(i, path)| {
                label_row(format!("Sprite {}", i + 1), path.clone())
            }))
            .spacing(4)
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

fn label_row(key: impl Into<String>, value: impl Into<String>) -> Element<'static, Message> {
    use iced::widget::Row;
    Row::new()
        .push(text(key.into()).width(150))
        .push(text(value.into()))
        .spacing(8)
        .into()
}
