use iced::widget::{button, column, container, row, text};
use iced::{Color, Element};

use dispel_core::modding::{ChangeAction, ChangeOp, Value};

use crate::editors::mod_packager::ModPackagerMessage;
use crate::message::{Message, MessageExt};

/// Render the expanded diff panel for a single changelog entry.
pub fn view<'a>(action: &'a ChangeAction) -> Element<'a, Message> {
    match &action.op {
        ChangeOp::FieldDelta {
            record_id,
            field,
            old,
            new,
        } => field_delta_panel(action, *record_id, field, old, new),
        ChangeOp::BinaryDelta { patch_bytes } => binary_delta_panel(action, patch_bytes),
        ChangeOp::FileReplace { content } => file_replace_panel(action, content),
        ChangeOp::FileAdd { content } => file_add_panel(action, content),
        ChangeOp::FileDelete => file_delete_panel(action),
    }
}

// ---------------------------------------------------------------------------
// FieldDelta
// ---------------------------------------------------------------------------

fn field_delta_panel<'a>(
    action: &'a ChangeAction,
    record_id: u32,
    field: &str,
    old: &Value,
    new: &Value,
) -> Element<'a, Message> {
    let header = text(format!(
        "{}  Record #{}: {}",
        action.file_path, record_id, field
    ))
    .size(11);

    let old_line = row![
        text("  - Old: ").size(11),
        text(format_display_value(old)).size(11)
            .color(Color::from_rgb(0.6, 0.2, 0.2)),
    ]
    .spacing(0);

    let new_line = row![
        text("  + New: ").size(11),
        text(format_display_value(new)).size(11)
            .color(Color::from_rgb(0.2, 0.5, 0.2)),
    ]
    .spacing(0);

    let inner = column![header, old_line, new_line].spacing(2);

    container(inner)
        .padding(6)
        .style(container::bordered_box)
        .into()
}

// ---------------------------------------------------------------------------
// BinaryDelta
// ---------------------------------------------------------------------------

fn binary_delta_panel<'a>(
    action: &'a ChangeAction,
    patch_bytes: &[u8],
) -> Element<'a, Message> {
    let header = text(format!(
        "{}  —  Binary delta — {} byte patch",
        action.file_path,
        patch_bytes.len(),
    ))
    .size(11);

    let open_btn = button(text("Open Hex Diff").size(11))
        .style(button::secondary)
        .padding([4, 8])
        .on_press(Message::mod_packager(ModPackagerMessage::OpenHexDiff(
            action.id,
        )));

    let inner = column![header, open_btn].spacing(4);

    container(inner)
        .padding(6)
        .style(container::bordered_box)
        .into()
}

// ---------------------------------------------------------------------------
// FileReplace
// ---------------------------------------------------------------------------

fn file_replace_panel<'a>(
    action: &'a ChangeAction,
    content: &[u8],
) -> Element<'a, Message> {
    let header = text(format!(
        "{}  —  File replacement — {} bytes",
        action.file_path,
        content.len(),
    ))
    .size(11);

    let open_btn = button(text("Open Hex Diff").size(11))
        .style(button::secondary)
        .padding([4, 8])
        .on_press(Message::mod_packager(ModPackagerMessage::OpenHexDiff(
            action.id,
        )));

    let inner = column![header, open_btn].spacing(4);

    container(inner)
        .padding(6)
        .style(container::bordered_box)
        .into()
}

// ---------------------------------------------------------------------------
// FileAdd
// ---------------------------------------------------------------------------

fn file_add_panel<'a>(action: &'a ChangeAction, content: &[u8]) -> Element<'a, Message> {
    let header = text(format!(
        "{}  —  New file — {} bytes",
        action.file_path,
        content.len(),
    ))
    .size(11);

    container(header)
        .padding(6)
        .style(container::bordered_box)
        .into()
}

// ---------------------------------------------------------------------------
// FileDelete
// ---------------------------------------------------------------------------

fn file_delete_panel<'a>(action: &'a ChangeAction) -> Element<'a, Message> {
    let header = text(format!("{}  —  File deletion", action.file_path)).size(11);

    container(header)
        .padding(6)
        .style(container::bordered_box)
        .into()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Format a [`Value`] for display in the diff panel.
///
/// Strings are wrapped in quotes to visually distinguish them from the diff
/// markers.
fn format_display_value(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{s}\""),
        Value::I64(i) => i.to_string(),
        Value::F64(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Bytes(b) => format!("<{} bytes>", b.len()),
        Value::Null => "(null)".to_owned(),
    }
}
