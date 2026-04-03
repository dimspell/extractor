use crate::message::Message;
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

pub fn labeled_select<'a, T: Clone + ToString + 'static>(
    label: &'a str,
    value: T,
    options: Vec<T>,
    on_change: impl Fn(T) -> Message + 'a,
) -> Element<'a, Message>
where
    T: PartialEq,
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
        move |path| Message::FileSelected {
            field: field.clone(),
            path,
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
        move |path| Message::FileSelected {
            field: field.clone(),
            path,
        },
    )
}

pub async fn run_command(exe: String, args: Vec<String>) -> Result<String, String> {
    use tokio::process::Command;
    let output = Command::new(&exe)
        .args(&args)
        .output()
        .await
        .map_err(|e| format!("Failed to spawn '{}': {}", exe, e))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let mut result = String::new();
    if !stdout.is_empty() {
        result.push_str(&stdout);
    }
    if !stderr.is_empty() {
        result.push_str(&stderr);
    }
    if output.status.success() {
        Ok(result)
    } else {
        Err(format!(
            "Exit code {}.\n{}",
            output.status.code().unwrap_or(-1),
            result
        ))
    }
}
pub fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        return path.to_string();
    }
    let half = (max_len.saturating_sub(3)) / 2;
    if half == 0 {
        return path.chars().take(max_len).collect();
    }
    format!("{}...{}", &path[..half], &path[path.len() - half..])
}
