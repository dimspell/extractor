use std::collections::BTreeMap;

use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Color, Element, Fill};

use dispel_core::modding::{ChangeAction, ChangeOp, Value};

use crate::app::App;
use crate::editors::mod_packager::ModPackagerMessage;
use crate::message::{Message, MessageExt};
use gui_widgets::lucide::{icon_char, LUCIDE_FONT};
use lucide_icons::Icon;

/// Render the Review tab — changes grouped by file path with per-field diffs
/// and revert buttons.
pub fn view(app: &App) -> Element<'_, Message> {
    let state = &app.state.editors.mod_packager_editor;

    // Group changes by file_path.
    let mut grouped: BTreeMap<&str, Vec<&ChangeAction>> = BTreeMap::new();
    for change in &state.selected_changes {
        grouped
            .entry(change.file_path.as_str())
            .or_default()
            .push(change);
    }

    if grouped.is_empty() {
        return container(text("No changes to review. Record edits to see them here.").size(12))
            .padding(40)
            .center_x(Fill)
            .center_y(Fill)
            .into();
    }

    let mut sections = Vec::new();
    for (file_path, changes) in &grouped {
        let header = row![
            text(icon_char(Icon::FolderOpen)).font(LUCIDE_FONT).size(13),
            text(format!(" {} — {} change(s)", file_path, changes.len())).size(13),
        ]
        .spacing(2);
        let mut section = column![header].spacing(6).padding(8);

        for action in changes.iter() {
            let card: Element<'_, Message> = change_card(action);
            section = section.push(card);
        }

        sections.push(
            container(section)
                .width(Fill)
                .style(container::bordered_box)
                .padding(6)
                .into(),
        );
    }

    let content = column(sections).spacing(12).padding(8);

    scrollable(container(content).width(Fill).height(Fill)).into()
}

fn change_card<'a>(action: &'a ChangeAction) -> Element<'a, Message> {
    match &action.op {
        ChangeOp::FieldDelta {
            record_id,
            field,
            old,
            new,
        } => field_card(action, *record_id, field, old, new),
        ChangeOp::BinaryDelta { patch_bytes } => {
            let body = column![
                text("Binary delta").size(12),
                text(format!("  {} byte patch", patch_bytes.len())).size(11),
            ]
            .spacing(2);
            container(body).padding(6).width(Fill).into()
        }
        ChangeOp::FileReplace { content } => {
            let body = column![
                text("File replacement").size(12),
                text(format!("  {} bytes", content.len())).size(11),
            ]
            .spacing(2);
            container(body).padding(6).width(Fill).into()
        }
        ChangeOp::FileAdd { content } => {
            let body = column![
                text("New file").size(12),
                text(format!("  {} bytes", content.len())).size(11),
            ]
            .spacing(2);
            container(body).padding(6).width(Fill).into()
        }
        ChangeOp::FileDelete => container(text("File deletion").size(11))
            .padding(6)
            .width(Fill)
            .into(),
    }
}

fn field_card<'a>(
    action: &'a ChangeAction,
    record_id: u32,
    field: &'a str,
    old: &'a Value,
    new: &'a Value,
) -> Element<'a, Message> {
    let header = text(format!("Record #{record_id}: {field}")).size(12);

    let old_line = row![
        text("  - Old: ")
            .color(Color::from_rgb(0.6, 0.2, 0.2))
            .size(11),
        text(format_display_value(old))
            .color(Color::from_rgb(0.6, 0.2, 0.2))
            .size(11),
    ];

    let new_line = row![
        text("  + New: ")
            .color(Color::from_rgb(0.2, 0.5, 0.2))
            .size(11),
        text(format_display_value(new))
            .color(Color::from_rgb(0.2, 0.5, 0.2))
            .size(11),
    ];

    let revert_btn = button(text("Revert").size(11))
        .padding([2, 8])
        .style(button::danger)
        .on_press(Message::mod_packager(ModPackagerMessage::RevertChange(
            action.id,
        )));

    let body = column![
        header,
        old_line,
        new_line,
        row![revert_btn].spacing(4).padding([4, 0]),
    ]
    .spacing(2);

    container(body).padding(6).width(Fill).into()
}

fn format_display_value(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{s}\""),
        other => other.to_string(),
    }
}
