use crate::message::{system::SystemMessage, Message};
use crate::style;
use iced::widget::{button, row, rule::Rule, space::Space, text, text_input};
use iced::{Element, Length, Task};

pub fn horizontal_space() -> Space {
    Space::default().width(Length::Fill)
}

pub fn vertical_space() -> Space {
    Space::default().height(Length::Fill)
}

pub fn horizontal_rule(height: u16) -> Rule<'static> {
    iced::widget::rule::horizontal(height as f32)
}

pub fn labeled_input<'a>(
    label: &'a str,
    value: &'a str,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    row![
        text(label).size(13).width(140),
        text_input("", value)
            .on_input(on_change)
            .padding(8)
            .size(13)
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

pub fn labeled_select<'a, T>(
    label: &'a str,
    value: T,
    options: Vec<T>,
    on_change: impl Fn(T) -> Message + 'a,
) -> Element<'a, Message>
where
    T: Clone + ToString + PartialEq + 'static,
{
    use iced::widget::pick_list;
    row![
        text(label).size(13).width(140),
        pick_list(options, Some(value), on_change)
            .padding(6)
            .text_size(13)
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

pub fn labeled_file_row<'a>(
    label: &'a str,
    value: &'a str,
    on_change: impl Fn(String) -> Message + 'a,
    browse_msg: Message,
) -> Element<'a, Message> {
    row![
        text(label).size(13).width(140),
        text_input("", value)
            .on_input(on_change)
            .padding(8)
            .size(13),
        button(text("…").size(12))
            .padding([6, 10])
            .on_press(browse_msg)
            .style(style::browse_button),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

pub fn browse_file(field: &str) -> Task<Message> {
    let field = field.to_string();
    Task::perform(
        async move {
            rfd::AsyncFileDialog::new()
                .pick_file()
                .await
                .map(|h| h.path().to_path_buf())
        },
        move |path| {
            Message::System(SystemMessage::FileSelected {
                field: field.clone(),
                path: Some(path.unwrap_or_default()),
            })
        },
    )
}

pub fn browse_folder(field: &str) -> Task<Message> {
    let field = field.to_string();
    Task::perform(
        async move {
            rfd::AsyncFileDialog::new()
                .pick_folder()
                .await
                .map(|h| h.path().to_path_buf())
        },
        move |path| {
            Message::System(SystemMessage::FileSelected {
                field: field.clone(),
                path: Some(path.unwrap_or_default()),
            })
        },
    )
}

pub fn truncate_path(path: &str, max_len: usize) -> String {
    let char_count = path.chars().count();
    if char_count <= max_len {
        return path.to_string();
    }
    let half = (max_len.saturating_sub(3)) / 2;
    if half == 0 {
        return path.chars().take(max_len).collect();
    }
    let start: String = path.chars().take(half).collect();
    let end: String = path.chars().skip(char_count - half).collect();
    format!("{}...{}", start, end)
}
