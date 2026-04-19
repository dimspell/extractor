use crate::app::App;
use crate::message::{startpage::StartPageMessage, Message};
use crate::style;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Background, Border, Color, Element, Fill, Font, Length};

fn card_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(0x22, 0x18, 0x14))),
        border: Border {
            color: Color::from_rgb8(0x5d, 0x40, 0x37),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

fn path_input_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(0x10, 0x0c, 0x08))),
        border: Border {
            color: Color::from_rgb8(0x4a, 0x33, 0x28),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

fn recent_item_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let base_bg = Color::from_rgb8(0x2d, 0x1f, 0x1b);
    let hover_bg = Color::from_rgb8(0x3d, 0x2b, 0x22);
    button::Style {
        background: Some(Background::Color(match status {
            button::Status::Hovered | button::Status::Pressed => hover_bg,
            _ => base_bg,
        })),
        border: Border {
            color: Color::from_rgb8(0x4a, 0x33, 0x28),
            width: 1.0,
            radius: 4.0.into(),
        },
        text_color: Color::from_rgb8(0xc4, 0xa8, 0x82),
        ..Default::default()
    }
}

fn divider_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(0x4a, 0x33, 0x28))),
        ..Default::default()
    }
}

impl App {
    pub fn view_start_page(&self) -> Element<'_, Message> {
        let sp = |msg: StartPageMessage| -> Message { Message::StartPage(msg) };

        let path = &self.start_page_input;
        let can_continue = !path.is_empty() && std::path::Path::new(path).exists();

        // ── Header ──────────────────────────────────────────────────────────
        let header = column![
            text("DISPEL EXTRACTOR")
                .size(34)
                .font(Font::MONOSPACE)
                .style(|_theme: &iced::Theme| text::Style {
                    color: Some(Color::from_rgb8(0xd4, 0xb4, 0x83)),
                }),
            text("Select your Dispel game installation folder to begin.")
                .size(14)
                .style(style::subtle_text),
        ]
        .spacing(10)
        .align_x(iced::Alignment::Center);

        // ── Path input row ───────────────────────────────────────────────────
        let path_row = row![
            container(
                text_input("e.g. /Users/you/Games/Dispel", path)
                    .on_input(|s| Message::StartPage(StartPageMessage::PathInputChanged(s)))
                    .padding([10, 14])
                    .size(13)
                    .font(Font::MONOSPACE)
            )
            .style(path_input_style)
            .width(Fill),
            button(text("Browse").size(13))
                .on_press(sp(StartPageMessage::Browse))
                .padding([10, 18])
                .style(style::browse_button),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        // ── Continue button ──────────────────────────────────────────────────
        let continue_btn = if can_continue {
            button(text("Continue  →").size(15).font(Font::MONOSPACE))
                .on_press(sp(StartPageMessage::Continue))
                .padding([12, 24])
                .width(Fill)
                .style(style::run_button)
        } else {
            button(text("Continue  →").size(15).font(Font::MONOSPACE))
                .padding([12, 24])
                .width(Fill)
                .style(style::run_button_disabled)
        };

        // ── Path form ────────────────────────────────────────────────────────
        let path_form = column![
            text("Game Installation Path")
                .size(11)
                .style(style::subtle_text),
            path_row,
            continue_btn,
        ]
        .spacing(10);

        // ── Recent paths ─────────────────────────────────────────────────────
        let recent = &self.state.workspace.recent_game_paths;
        let recent_section: Element<'_, Message> = if recent.is_empty() {
            column![].into()
        } else {
            let items: Vec<Element<'_, Message>> = recent
                .iter()
                .map(|p| {
                    let display = p.to_string_lossy().to_string();
                    let label = p
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| display.clone());
                    button(
                        column![
                            text(label).size(12).font(Font::MONOSPACE),
                            text(display).size(10).style(style::subtle_text),
                        ]
                        .spacing(2),
                    )
                    .on_press(sp(StartPageMessage::SelectRecentPath(p.clone())))
                    .width(Fill)
                    .padding([8, 12])
                    .style(recent_item_style)
                    .into()
                })
                .collect();

            column![
                container(text(""))
                    .style(divider_style)
                    .width(Fill)
                    .height(1),
                text("Recent Paths").size(11).style(style::subtle_text),
                scrollable(column(items).spacing(4)),
            ]
            .spacing(12)
            .into()
        };

        // ── Card ─────────────────────────────────────────────────────────────
        let card = container(
            column![header, path_form, recent_section,]
                .spacing(28)
                .width(Length::Fixed(540.0)),
        )
        .style(card_style)
        .padding(48);

        // ── Full screen centered layout ───────────────────────────────────────
        container(card)
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill)
            .style(style::root_container)
            .into()
    }
}
