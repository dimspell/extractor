use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Fill, Length};

use crate::components::utils::horizontal_space;

use crate::app::App;
use crate::editors::mod_packager::state::ModManagerTab;
use crate::editors::mod_packager::ModPackagerMessage;
use crate::message::{Message, MessageExt};

mod conflicts;
mod detail;
mod library;

pub fn view(app: &App) -> Element<'_, Message> {
    let state = &app.state.mod_packager_editor;

    let header = build_header(
        state.tab,
        state.workspace_root.is_some(),
        state.conflicts.len(),
    );
    let banner = recording_banner(app);

    let body: Element<'_, Message> = match state.tab {
        ModManagerTab::Library => library::view(app),
        ModManagerTab::Detail => detail::view(app),
        ModManagerTab::Conflicts => conflicts::view(app),
    };

    let status = text(state.status_msg.as_str()).size(12);

    let mut col = column![header].spacing(12).padding(16).width(Fill).height(Fill);
    if let Some(b) = banner {
        col = col.push(b);
    }
    col = col.push(body).push(status);

    container(col).width(Fill).height(Fill).into()
}

fn recording_banner(app: &App) -> Option<Element<'_, Message>> {
    let session = app.state.recording.as_ref()?;
    let pending_suffix = if session.pending.is_empty() {
        String::new()
    } else {
        format!(", {} pending", session.pending.len())
    };
    let label = text(format!(
        "● Recording into `{}` — {} change(s) captured{}",
        session.mod_name, session.recorded_count, pending_suffix
    ))
    .size(12);
    let stop = button(text("Stop").size(12))
        .padding([4, 12])
        .style(button::danger)
        .on_press(Message::mod_packager(ModPackagerMessage::StopRecording));
    Some(
        container(
            row![label, horizontal_space().width(Length::Fill), stop]
                .spacing(8)
                .align_y(Alignment::Center),
        )
        .padding(8)
        .style(container::bordered_box)
        .into(),
    )
}

fn build_header(
    active: ModManagerTab,
    has_workspace: bool,
    conflict_count: usize,
) -> Element<'static, Message> {
    let tab_btn = |label: String, target: ModManagerTab| {
        let btn = button(text(label).size(12)).padding([4, 12]);
        let btn = if active == target {
            btn.style(button::primary)
        } else {
            btn.style(button::secondary)
        };
        btn.on_press(Message::mod_packager(ModPackagerMessage::TabSelected(
            target,
        )))
    };
    let conflicts_label = if conflict_count > 0 {
        format!("Conflicts ({conflict_count})")
    } else {
        "Conflicts".to_owned()
    };

    let open_workspace_btn = button(
        text(if has_workspace {
            "Change Workspace…"
        } else {
            "Open Workspace…"
        })
        .size(12),
    )
    .padding([4, 12])
    .on_press(Message::mod_packager(ModPackagerMessage::OpenWorkspace));

    row![
        tab_btn("Library".to_owned(), ModManagerTab::Library),
        tab_btn("Detail".to_owned(), ModManagerTab::Detail),
        tab_btn(conflicts_label, ModManagerTab::Conflicts),
        horizontal_space().width(Length::Fill),
        open_workspace_btn,
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}
