//! Shared UI helpers for the save file viewer sections.

use iced::widget::{Row, container, text};
use iced::{Element, Fill};

use crate::message::Message;

/// Section header label (e.g. "Core Attributes", "Combat Stats").
pub fn section_header(label: &str) -> Element<'static, Message> {
    container(text(label.to_string()).size(16))
        .padding([8, 0])
        .width(Fill)
        .into()
}

/// A key-value row with fixed-width label.
pub fn label_row(key: impl Into<String>, value: impl Into<String>) -> Element<'static, Message> {
    Row::new()
        .push(text(key.into()).width(150))
        .push(text(value.into()))
        .spacing(8)
        .into()
}
