use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Fill, Length};

use crate::components::utils::horizontal_space;

use crate::app::App;
use crate::editors::mod_packager::state::ModManagerTab;
use crate::editors::mod_packager::ModPackagerMessage;
use crate::message::{Message, MessageExt};

mod detail;
mod library;

pub fn view(app: &App) -> Element<'_, Message> {
    let state = &app.state.mod_packager_editor;

    let header = build_header(state.tab, state.workspace_root.is_some());

    let body: Element<'_, Message> = match state.tab {
        ModManagerTab::Library => library::view(app),
        ModManagerTab::Detail => detail::view(app),
    };

    let status = text(state.status_msg.as_str()).size(12);

    let content = column![header, body, status]
        .spacing(12)
        .padding(16)
        .width(Fill)
        .height(Fill);

    container(content).width(Fill).height(Fill).into()
}

fn build_header(active: ModManagerTab, has_workspace: bool) -> Element<'static, Message> {
    let tab_btn = |label: &str, target: ModManagerTab| {
        let btn = button(text(label.to_owned()).size(12)).padding([4, 12]);
        let btn = if active == target {
            btn.style(button::primary)
        } else {
            btn.style(button::secondary)
        };
        btn.on_press(Message::mod_packager(ModPackagerMessage::TabSelected(
            target,
        )))
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
        tab_btn("Library", ModManagerTab::Library),
        tab_btn("Detail", ModManagerTab::Detail),
        horizontal_space().width(Length::Fill),
        open_workspace_btn,
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}
