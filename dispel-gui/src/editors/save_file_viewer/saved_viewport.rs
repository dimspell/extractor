//! Dedicated view for the active map viewport stored in a save file.

use crate::editors::save_file_viewer::SaveFileViewerState;
use crate::message::Message;
use iced::widget::{column, container, text};
use iced::{Element, Fill};

pub fn view<'a>(state: &'a SaveFileViewerState) -> Element<'a, Message> {
    match state.map_preview.as_ref() {
        Some(preview) => crate::editors::save_file_viewer::map_preview::view_preview(preview),
        None => {
            let map_id = state
                .save_file
                .as_ref()
                .map(|save| save.post_maps.all_map_ini_id.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            container(
                column![
                    text("Loading saved viewport…").size(13),
                    text(format!("Active map ID: {map_id}")).size(11),
                ]
                .spacing(6)
                .padding(16),
            )
            .width(Fill)
            .height(Fill)
            .into()
        }
    }
}
