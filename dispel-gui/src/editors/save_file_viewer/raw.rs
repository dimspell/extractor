use iced::widget::{container, scrollable, text, Column, Row};
use iced::{Element, Fill};

use crate::editors::save_file_viewer::state::SaveFileViewerState;
use crate::editors::save_file_viewer::SaveFileViewerMessage;
use crate::message::Message;
use crate::message::MessageExt;

/// Raw section: embedded hex editors for unknown blocks.
pub fn view<'a>(state: &'a SaveFileViewerState) -> Element<'a, Message> {
    if state.raw_hex_viewers.is_empty() {
        return container(text("No raw sections available"))
            .width(Fill)
            .height(Fill)
            .padding(16)
            .into();
    }

    let mut col = Column::new().spacing(8).padding(16);

    for (i, viewer) in state.raw_hex_viewers.iter().enumerate() {
        let section = container(
            Column::new()
                .spacing(4)
                .push(
                    container(text(viewer.label.to_string()).size(14))
                        .padding([8, 0])
                        .width(Fill),
                )
                .push(
                    hexedit::view(
                        &viewer.state,
                        &hexedit::HexEditorConfig {
                            can_save: false,
                            ..hexedit::HexEditorConfig::default()
                        },
                    )
                    .map(move |msg| {
                        Message::save_file_viewer(SaveFileViewerMessage::HexViewer(i, msg))
                    }),
                ),
        )
        .width(Fill);
        col = col.push(section);
    }

    scrollable(col).into()
}
