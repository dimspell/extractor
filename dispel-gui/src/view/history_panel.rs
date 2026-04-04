use crate::edit_history::EditHistory;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill};

pub fn view_history_panel(history: &EditHistory) -> Element<'_, crate::message::Message> {
    use crate::message::Message;

    let header = row![
        text("Edit History").size(14),
        crate::utils::horizontal_space(),
        button(text("×").size(16))
            .on_press(Message::ToggleHistoryPanel)
            .style(crate::style::chip),
    ]
    .padding([8, 12]);

    let undo_items: Vec<Element<Message>> = history
        .undo_stack()
        .iter()
        .enumerate()
        .map(|(idx, action)| {
            let label = format!("{}. {}", idx + 1, action.display_text());
            button(text(label).size(11).font(iced::Font::MONOSPACE))
                .width(Fill)
                .on_press(Message::Undo)
                .style(crate::style::chip)
                .into()
        })
        .collect();

    let redo_items: Vec<Element<Message>> = history
        .redo_stack()
        .iter()
        .enumerate()
        .map(|(idx, action)| {
            let label = format!("{}. {}", idx + 1, action.display_text());
            button(text(label).size(11).font(iced::Font::MONOSPACE))
                .width(Fill)
                .on_press(Message::Redo)
                .style(crate::style::chip)
                .into()
        })
        .collect();

    let undo_section =
        column![text("Undo Stack").size(12).style(crate::style::subtle_text),].padding([0, 8]);

    let redo_section =
        column![text("Redo Stack").size(12).style(crate::style::subtle_text),].padding([0, 8]);

    let content = column![
        undo_section,
        column(undo_items).spacing(2),
        crate::utils::horizontal_space().height(10),
        redo_section,
        column(redo_items).spacing(2),
    ]
    .spacing(4);

    let scroll = scrollable(content).height(Fill);

    container(column![header, crate::utils::horizontal_rule(1), scroll,].spacing(0))
        .height(Fill)
        .width(280)
        .style(crate::style::sidebar_container)
        .into()
}
