use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Fill, Length};

use dispel_core::modding::InstalledMod;

use crate::app::App;
use crate::components::loading_state::LoadingState;
use crate::components::utils::horizontal_space;
use crate::editors::mod_packager::ModPackagerMessage;
use crate::message::{Message, MessageExt};

pub fn view(app: &App) -> Element<'_, Message> {
    let state = &app.state.mod_packager_editor;
    let busy = matches!(state.loading_state, LoadingState::Loading);

    if state.workspace_root.is_none() {
        return empty_workspace();
    }

    let toolbar = row![
        action_btn("New Mod", ModPackagerMessage::CreateMod, busy),
        action_btn("Import .zip", ModPackagerMessage::ImportZip, busy),
        horizontal_space().width(Length::Fill),
        action_btn("Apply to Game", ModPackagerMessage::Apply, busy)
            .style(button::primary),
        action_btn("Revert to Vanilla", ModPackagerMessage::Revert, busy)
            .style(button::danger),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let recording_slug = app
        .state
        .recording
        .as_ref()
        .map(|s| s.mod_slug.clone());
    let rows: Vec<Element<'_, Message>> = state
        .mods
        .iter()
        .map(|m| mod_row(m, busy, recording_slug.as_deref() == Some(m.slug.as_str())))
        .collect();

    let list: Element<'_, Message> = if rows.is_empty() {
        container(text("No mods installed yet. Click \"New Mod\" or import a .zip.").size(12))
            .padding(20)
            .into()
    } else {
        scrollable(column(rows).spacing(4).padding(4).width(Fill))
            .height(Length::Fill)
            .into()
    };

    column![toolbar, list]
        .spacing(8)
        .height(Length::Fill)
        .width(Fill)
        .into()
}

fn empty_workspace() -> Element<'static, Message> {
    container(
        column![
            text("No workspace selected.").size(14),
            text("Pick a folder where dispel-gui will store mods, the load order, and pristine-game snapshots.")
                .size(12),
            button(text("Open Workspace…").size(13))
                .on_press(Message::mod_packager(ModPackagerMessage::OpenWorkspace)),
        ]
        .spacing(12)
        .align_x(Alignment::Center),
    )
    .padding(40)
    .center_x(Fill)
    .center_y(Fill)
    .into()
}

fn mod_row<'a>(m: &'a InstalledMod, busy: bool, is_recording: bool) -> Element<'a, Message> {
    let toggle_label = if m.enabled { "Disable" } else { "Enable" };
    let toggle = action_btn(
        toggle_label,
        ModPackagerMessage::ToggleEnabled(m.slug.clone()),
        busy,
    );

    let up = action_btn("↑", ModPackagerMessage::MoveUp(m.slug.clone()), busy || !m.enabled);
    let down = action_btn(
        "↓",
        ModPackagerMessage::MoveDown(m.slug.clone()),
        busy || !m.enabled,
    );

    let record_btn = if is_recording {
        action_btn("Stop Rec", ModPackagerMessage::StopRecording, busy)
            .style(button::danger)
    } else {
        action_btn(
            "Record",
            ModPackagerMessage::StartRecording(m.slug.clone()),
            busy,
        )
    };

    let edit = action_btn(
        "Edit",
        ModPackagerMessage::SelectMod(m.slug.clone()),
        busy,
    );
    let export = action_btn(
        "Export",
        ModPackagerMessage::ExportZip(m.slug.clone()),
        busy,
    );
    let delete = action_btn("Delete", ModPackagerMessage::DeleteMod(m.slug.clone()), busy)
        .style(button::danger);

    let badge = text(if m.enabled { "● enabled" } else { "○ disabled" })
        .size(11)
        .width(Length::Fixed(80.0));

    let title = text(m.manifest.name.as_str()).size(13);
    let subtitle = text(format!(
        "{}  •  {} change(s){}",
        m.slug,
        m.change_count,
        if m.manifest.version.is_empty() {
            String::new()
        } else {
            format!("  •  v{}", m.manifest.version)
        }
    ))
    .size(11);

    container(
        row![
            badge,
            column![title, subtitle].spacing(2).width(Length::Fill),
            up,
            down,
            toggle,
            record_btn,
            edit,
            export,
            delete,
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .padding(8)
    .style(container::bordered_box)
    .into()
}

fn action_btn(label: &str, msg: ModPackagerMessage, disabled: bool) -> button::Button<'_, Message> {
    let b = button(text(label.to_owned()).size(12)).padding([4, 8]);
    if disabled {
        b
    } else {
        b.on_press(Message::mod_packager(msg))
    }
}
