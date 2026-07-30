use iced::widget::{container, scrollable, text, Column};
use iced::Element;

use crate::editors::save_file_viewer::helpers::{label_row, section_header};
use crate::editors::save_file_viewer::state::SaveFileViewerState;
use crate::message::Message;

/// Identity section: player name, class, and unknown blocks as hex.
pub fn view<'a>(state: &'a SaveFileViewerState) -> Element<'a, Message> {
    let sf = match state.save_file.as_ref() {
        Some(sf) => sf,
        None => return container(text("No save file loaded")).into(),
    };

    let identity = &sf.character_identity;

    scrollable(
        Column::new()
            .push(section_header("Player Identity"))
            .push(label_row("Player Name", &identity.player_name))
            .push(label_row("Class Name", &identity.player_class_name))
            .push(label_row("Class ID", identity.player_class_id.to_string()))
            .push(
                Column::new()
                    .spacing(4)
                    .push(section_header("Unknown Block (96 bytes)"))
                    .push(hex_block(&identity.unknown_block)),
            )
            .push(
                Column::new()
                    .spacing(4)
                    .push(section_header("Unknown Large Data (4040 bytes)"))
                    .push(hex_block(&identity.unknown_data)),
            )
            .spacing(8)
            .padding(16),
    )
    .into()
}

/// Compact hex dump: first 64 bytes as hex pairs.
fn hex_block(data: &[u8]) -> Element<'static, Message> {
    let max_bytes = 64.min(data.len());
    let preview = data[..max_bytes]
        .chunks(16)
        .map(|chunk| {
            chunk
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n");

    let display = if data.len() > 64 {
        format!("{}\n\n{} bytes (showing first 64)", preview, data.len())
    } else {
        format!("{}\n\n{} bytes", preview, data.len())
    };

    container(text(display).size(11))
        .padding(8)
        .style(|_theme| container::Style {
            text_color: Some(iced::Color::from_rgb8(180, 180, 180)),
            ..container::Style::default()
        })
        .into()
}
