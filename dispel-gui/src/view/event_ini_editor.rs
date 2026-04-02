use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::{
    horizontal_rule, horizontal_space, labeled_input, labeled_select, vertical_space,
};
use dispel_core::EventType;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font, Length};

impl App {
    pub fn view_event_ini_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.event_ini_editor;

        let item_list: Vec<Element<Message>> = editor
            .filtered_events
            .iter()
            .enumerate()
            .map(|(idx, (_, event))| {
                let is_selected = editor.selected_idx == Some(idx);
                let label = format!(
                    "[{}] evt:{} prev:{} type:{} cnt:{}",
                    idx,
                    event.event_id,
                    event.previous_event_id,
                    format!("{:?}", event.event_type),
                    event.counter
                );

                let btn = button(text(label).size(11).font(Font::MONOSPACE))
                    .width(Fill)
                    .on_press(Message::EventIniOpSelectEvent(idx));

                if is_selected {
                    btn.style(style::active_chip).into()
                } else {
                    btn.style(style::chip).into()
                }
            })
            .collect();

        let item_scroll = scrollable(column(item_list).spacing(4)).height(Length::Fill);

        let mut detail_content: Vec<Element<Message>> = vec![
            text("Event Details").size(16).font(Font::MONOSPACE).into(),
            vertical_space().height(10).into(),
        ];

        if let Some(idx) = editor.selected_idx {
            if let Some((orig_idx, _event)) = editor.filtered_events.get(idx) {
                let orig = *orig_idx;

                detail_content.push(labeled_input(
                    "Event ID:",
                    &editor.edit_event_id,
                    move |v| Message::EventIniOpFieldChanged(orig, "event_id".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Previous Event ID:",
                    &editor.edit_previous_event_id,
                    move |v| Message::EventIniOpFieldChanged(orig, "previous_event_id".into(), v),
                ));

                let event_type_options = vec![
                    EventType::Unknown,
                    EventType::Conditional,
                    EventType::ContinueOnUnsatisfied,
                    EventType::ExecuteOnSatisfied,
                ];
                let event_type_value = if editor.edit_event_type.contains("Conditional") {
                    EventType::Conditional
                } else if editor.edit_event_type.contains("ContinueOnUnsatisfied") {
                    EventType::ContinueOnUnsatisfied
                } else if editor.edit_event_type.contains("ExecuteOnSatisfied") {
                    EventType::ExecuteOnSatisfied
                } else {
                    EventType::Unknown
                };
                detail_content.push(labeled_select(
                    "Event Type:",
                    event_type_value,
                    event_type_options,
                    move |v| {
                        Message::EventIniOpFieldChanged(
                            orig,
                            "event_type".into(),
                            format!("{:?}", v),
                        )
                    },
                ));

                detail_content.push(labeled_input(
                    "Event Filename:",
                    &editor.edit_event_filename,
                    move |v| Message::EventIniOpFieldChanged(orig, "event_filename".into(), v),
                ));
                detail_content.push(labeled_input("Counter:", &editor.edit_counter, move |v| {
                    Message::EventIniOpFieldChanged(orig, "counter".into(), v)
                }));
            }
        } else {
            detail_content.push(
                text("No event selected")
                    .size(13)
                    .style(style::subtle_text)
                    .into(),
            );
        }

        let detail_scroll = scrollable(column(detail_content).spacing(8)).height(Length::Fill);

        let detail_panel = container(detail_scroll)
            .padding(16)
            .width(380)
            .style(style::info_card);

        let item_list_header = row![
            text("Events").size(14),
            horizontal_space(),
            button(text("Scan"))
                .on_press(Message::EventIniOpLoadCatalog)
                .padding([5, 10])
                .style(style::run_button),
        ]
        .padding(10)
        .align_y(iced::Alignment::Center);

        let left_panel = column![
            container(item_list_header).style(style::grid_header_cell),
            item_scroll,
        ];

        let main_content = row![left_panel, detail_panel.width(Length::FillPortion(2)),]
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
                    button(text("Save Events"))
                        .on_press(Message::EventIniOpSave)
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
