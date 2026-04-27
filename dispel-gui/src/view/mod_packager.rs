use crate::app::App;
use crate::loading_state::LoadingState;
use crate::message::editor::mod_packager::ModPackagerMessage;
use crate::message::{Message, MessageExt};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Fill, Length};

impl App {
    pub fn view_mod_packager_tab(&self) -> Element<'_, Message> {
        let state = &self.state.mod_packager_editor;
        let is_loading = matches!(state.loading_state, LoadingState::Loading);

        // Metadata form
        let metadata = column![
            text("Mod Metadata").size(14),
            row![
                text("Name").width(Length::Fixed(80.0)),
                text_input("required", &state.metadata.name)
                    .on_input(|v| Message::mod_packager(ModPackagerMessage::NameChanged(v)))
                    .width(Fill),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                text("Version").width(Length::Fixed(80.0)),
                text_input("1.0.0", &state.metadata.version)
                    .on_input(|v| Message::mod_packager(ModPackagerMessage::VersionChanged(v)))
                    .width(Fill),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                text("Author").width(Length::Fixed(80.0)),
                text_input("", &state.metadata.author)
                    .on_input(|v| Message::mod_packager(ModPackagerMessage::AuthorChanged(v)))
                    .width(Fill),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                text("Description").width(Length::Fixed(80.0)),
                text_input("", &state.metadata.description)
                    .on_input(|v| Message::mod_packager(ModPackagerMessage::DescriptionChanged(v)))
                    .width(Fill),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(8)
        .padding([8, 0]);

        // File list header + add button
        let file_list_header = row![
            text(format!("Files ({})", state.selected_files.len()))
                .size(14)
                .width(Fill),
            button(text("Add Files…").size(12)).on_press_maybe(if is_loading {
                None
            } else {
                Some(Message::mod_packager(ModPackagerMessage::BrowseFiles))
            }),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let file_rows: Vec<Element<'_, Message>> = state
            .selected_files
            .iter()
            .enumerate()
            .map(|(i, path)| {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.to_string_lossy().into_owned());
                row![
                    text(name).size(12).width(Fill),
                    button(text("×").size(12))
                        .on_press_maybe(if is_loading {
                            None
                        } else {
                            Some(Message::mod_packager(ModPackagerMessage::RemoveFile(i)))
                        })
                        .padding([2, 6]),
                ]
                .spacing(8)
                .align_y(Alignment::Center)
                .into()
            })
            .collect();

        let file_list = scrollable(column(file_rows).spacing(4).padding([4, 0]).width(Fill))
            .height(Length::Fixed(200.0));

        // Export button + status
        let export_btn = button(text("Export .zip").size(13)).on_press_maybe(if is_loading {
            None
        } else {
            Some(Message::mod_packager(ModPackagerMessage::Export))
        });

        let status = text(&state.status_msg).size(12);

        let content = column![
            metadata,
            file_list_header,
            file_list,
            row![export_btn, status]
                .spacing(12)
                .align_y(Alignment::Center),
        ]
        .spacing(12)
        .padding(16)
        .width(Fill);

        container(content).width(Fill).height(Fill).into()
    }
}
