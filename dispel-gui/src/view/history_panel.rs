use crate::components::edit_history::EditHistory;
use crate::message::{Message, workspace::WorkspaceMessage};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill};

pub fn view_history_panel(history: &EditHistory) -> Element<'_, crate::message::Message> {
    let header = row![
        text("Edit History").size(14),
        crate::components::utils::horizontal_space(),
        button(text("×").size(16))
            .on_press(Message::Workspace(WorkspaceMessage::ToggleHistoryPanel))
            .style(crate::style::chip),
    ]
    .padding([8, 12]);

    // Entries are informational only. They are numbered for reference, not
    // clickable, because the panel has no way to jump to an arbitrary step.
    let undo_items: Vec<Element<Message>> = history
        .undo_stack()
        .iter()
        .enumerate()
        .map(|(idx, action)| {
            let label = format!("{}. {}", idx + 1, action.display_text());
            text(label).size(11).font(iced::Font::MONOSPACE).into()
        })
        .collect();

    let redo_items: Vec<Element<Message>> = history
        .redo_stack()
        .iter()
        .enumerate()
        .map(|(idx, action)| {
            let label = format!("{}. {}", idx + 1, action.display_text());
            text(label).size(11).font(iced::Font::MONOSPACE).into()
        })
        .collect();

    let undo_section =
        column![text("Undo Stack").size(12).style(crate::style::subtle_text),].padding([0, 8]);

    let redo_section =
        column![text("Redo Stack").size(12).style(crate::style::subtle_text),].padding([0, 8]);

    let content = column![
        undo_section,
        column(undo_items).spacing(2),
        crate::components::utils::horizontal_space().height(10),
        redo_section,
        column(redo_items).spacing(2),
    ]
    .spacing(4);

    let scroll = scrollable(content).height(Fill);

    container(column![header, crate::components::utils::horizontal_rule(1), scroll,].spacing(0))
        .height(Fill)
        .width(280)
        .style(crate::style::sidebar_container)
        .accessible_label("Edit history")
        .into()
}

#[cfg(all(test, feature = "iced_test"))]
mod tests {
    use super::*;
    use crate::components::edit_history::EditAction;
    use crate::message::system::SystemMessage;
    use iced_test::simulator;

    #[test]
    fn history_entries_do_not_emit_jump_messages() {
        let mut history = EditHistory::new();
        history.push(EditAction::FieldChange {
            record_idx: 0,
            field: "name".into(),
            old_value: "old".into(),
            new_value: "new".into(),
        });
        history.push(EditAction::FieldChange {
            record_idx: 1,
            field: "hp".into(),
            old_value: "10".into(),
            new_value: "20".into(),
        });

        let entry = "2. Changed name to \"new\"";
        let mut ui = simulator(view_history_panel(&history));
        ui.find(entry).expect("numbered entry must remain visible");
        let _ = ui.click(entry);
        let messages: Vec<Message> = ui.into_messages().collect();
        assert!(
            !messages.iter().any(|m| matches!(
                m,
                Message::System(SystemMessage::Undo) | Message::System(SystemMessage::Redo)
            )),
            "numbered entries are informational only, not jump buttons: {messages:?}"
        );
    }
}
