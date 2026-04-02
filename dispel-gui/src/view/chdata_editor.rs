use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::{horizontal_rule, horizontal_space, labeled_input, vertical_space};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font, Length};

impl App {
    pub fn view_chdata_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.chdata_editor;

        let mut detail_content: Vec<Element<Message>> = vec![
            text("Character Data").size(16).font(Font::MONOSPACE).into(),
            vertical_space().height(10).into(),
        ];

        if editor.catalog.is_some() {
            detail_content.push(labeled_input("Magic:", &editor.edit_magic, |v| {
                Message::ChDataOpFieldChanged("magic".into(), v)
            }));
            detail_content.push(labeled_input(
                "Values (comma-separated):",
                &editor.edit_values,
                |v| Message::ChDataOpFieldChanged("values".into(), v),
            ));
            detail_content.push(labeled_input(
                "Counts (comma-separated):",
                &editor.edit_counts,
                |v| Message::ChDataOpFieldChanged("counts".into(), v),
            ));
            detail_content.push(labeled_input("Total:", &editor.edit_total, |v| {
                Message::ChDataOpFieldChanged("total".into(), v)
            }));
        } else {
            detail_content.push(
                text("No data loaded")
                    .size(13)
                    .style(style::subtle_text)
                    .into(),
            );
        }

        let detail_scroll = scrollable(column(detail_content).spacing(8)).height(Length::Fill);

        let detail_panel = container(detail_scroll)
            .padding(16)
            .width(Fill)
            .style(style::info_card);

        let toolbar = row![
            text("ChData.db").size(14),
            horizontal_space(),
            button(text("Scan"))
                .on_press(Message::ChDataOpLoadCatalog)
                .padding([5, 10])
                .style(style::run_button),
        ]
        .padding(10)
        .align_y(iced::Alignment::Center);

        let main_content = column![
            container(toolbar).style(style::grid_header_cell),
            detail_panel,
        ]
        .spacing(0)
        .height(Length::Fill);

        column![
            horizontal_rule(1),
            main_content,
            container(
                row![
                    text(&editor.status_msg).size(13).style(style::subtle_text),
                    horizontal_space(),
                    if editor.is_loading {
                        Element::from(text("Loading...").size(13))
                    } else {
                        Element::from(text(""))
                    },
                    horizontal_space().width(20),
                    button(text("Save ChData"))
                        .on_press(Message::ChDataOpSave)
                        .style(style::commit_button),
                ]
                .padding([10, 20])
                .align_y(iced::Alignment::Center),
            )
            .width(Fill)
            .style(style::status_bar),
        ]
        .spacing(0)
        .height(Length::Fill)
        .into()
    }
}
