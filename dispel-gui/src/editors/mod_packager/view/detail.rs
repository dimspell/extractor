use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Fill, Length};

use dispel_core::modding::{ChangeAction, ChangeOp};

use crate::app::App;
use crate::components::loading_state::LoadingState;
use crate::components::utils::horizontal_space;
use crate::editors::mod_packager::ModPackagerMessage;
use crate::message::{Message, MessageExt};

pub fn view(app: &App) -> Element<'_, Message> {
    let state = &app.state.mod_packager_editor;
    let busy = matches!(state.loading_state, LoadingState::Loading);

    let Some(slug) = state.selected_slug.as_deref() else {
        return placeholder();
    };

    let manifest_section = manifest_form(state);
    let changelog_section = changelog_panel(&state.selected_changes);

    let is_recording_this = app
        .state
        .recording
        .as_ref()
        .map(|s| s.mod_slug == slug)
        .unwrap_or(false);
    let record_btn = if is_recording_this {
        action_btn("Stop Recording", ModPackagerMessage::StopRecording, busy)
            .style(button::danger)
    } else {
        action_btn(
            "Start Recording",
            ModPackagerMessage::StartRecording(slug.to_owned()),
            busy,
        )
        .style(button::primary)
    };
    let actions = row![
        save_button(state.edit_dirty, busy),
        record_btn,
        action_btn("Export .zip", ModPackagerMessage::ExportZip(slug.to_owned()), busy),
        horizontal_space().width(Length::Fill),
        action_btn(
            "Delete",
            ModPackagerMessage::DeleteMod(slug.to_owned()),
            busy,
        )
        .style(button::danger),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    column![manifest_section, actions, changelog_section]
        .spacing(12)
        .width(Fill)
        .height(Length::Fill)
        .into()
}

fn placeholder() -> Element<'static, Message> {
    container(text("Select a mod from the Library to edit its details.").size(12))
        .padding(40)
        .center_x(Fill)
        .center_y(Fill)
        .into()
}

fn manifest_form(state: &super::super::state::ModPackagerState) -> Element<'_, Message> {
    let labelled = |label: &'static str, value: &str, msg: fn(String) -> ModPackagerMessage| {
        let input = text_input("", value)
            .on_input(move |v| Message::mod_packager(msg(v)))
            .width(Fill);
        row![text(label).width(Length::Fixed(90.0)), input]
            .spacing(8)
            .align_y(Alignment::Center)
    };

    column![
        text("Manifest").size(14),
        labelled("Name", &state.edit_name, ModPackagerMessage::NameChanged),
        labelled(
            "Version",
            &state.edit_version,
            ModPackagerMessage::VersionChanged,
        ),
        labelled(
            "Author",
            &state.edit_author,
            ModPackagerMessage::AuthorChanged,
        ),
        labelled(
            "Description",
            &state.edit_description,
            ModPackagerMessage::DescriptionChanged,
        ),
    ]
    .spacing(6)
    .into()
}

fn save_button(dirty: bool, busy: bool) -> button::Button<'static, Message> {
    let label = if dirty { "Save" } else { "Saved" };
    let b = button(text(label).size(12)).padding([4, 12]);
    if busy || !dirty {
        b
    } else {
        b.style(button::primary)
            .on_press(Message::mod_packager(ModPackagerMessage::SaveManifest))
    }
}

fn changelog_panel(changes: &[ChangeAction]) -> Element<'_, Message> {
    let header = text(format!("Change Log ({})", changes.len())).size(14);

    let body: Element<'_, Message> = if changes.is_empty() {
        text(
            "No changes recorded yet. Edits captured via Recording mode (Phase 4) will land here.",
        )
        .size(11)
        .into()
    } else {
        let rows: Vec<Element<'_, Message>> =
            changes.iter().enumerate().map(|(i, a)| change_row(i, a)).collect();
        scrollable(column(rows).spacing(4).padding(4).width(Fill))
            .height(Length::Fill)
            .into()
    };

    column![header, body]
        .spacing(6)
        .height(Length::Fill)
        .width(Fill)
        .into()
}

fn change_row(index: usize, action: &ChangeAction) -> Element<'_, Message> {
    let summary = describe(action);
    let title = text(format!("#{}  {}", index + 1, summary)).size(12);
    let path = text(action.file_path.as_str()).size(10);
    let desc = if action.description.is_empty() {
        None
    } else {
        Some(text(action.description.as_str()).size(11))
    };

    let mut col = column![title, path].spacing(2);
    if let Some(d) = desc {
        col = col.push(d);
    }

    container(col).padding(6).style(container::bordered_box).into()
}

fn describe(action: &ChangeAction) -> String {
    match &action.op {
        ChangeOp::FieldDelta {
            record_id,
            field,
            new,
            ..
        } => format!(
            "FieldDelta  record #{record_id}.{field} = {}",
            preview_value(new)
        ),
        ChangeOp::BinaryDelta { patch_bytes } => {
            format!("BinaryDelta  ({} byte patch)", patch_bytes.len())
        }
        ChangeOp::FileReplace { content } => {
            format!("FileReplace  ({} bytes)", content.len())
        }
        ChangeOp::FileAdd { content } => format!("FileAdd  ({} bytes)", content.len()),
        ChangeOp::FileDelete => "FileDelete".to_owned(),
    }
}

fn preview_value(value: &dispel_core::modding::Value) -> String {
    use dispel_core::modding::Value;
    match value {
        Value::String(s) if s.len() > 40 => format!("\"{}…\"", &s[..40]),
        Value::String(s) => format!("\"{s}\""),
        Value::I64(i) => i.to_string(),
        Value::F64(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Bytes(b) => format!("<{} bytes>", b.len()),
        Value::Null => "null".to_owned(),
    }
}

fn action_btn(label: &str, msg: ModPackagerMessage, disabled: bool) -> button::Button<'_, Message> {
    let b = button(text(label.to_owned()).size(12)).padding([4, 8]);
    if disabled {
        b
    } else {
        b.on_press(Message::mod_packager(msg))
    }
}
