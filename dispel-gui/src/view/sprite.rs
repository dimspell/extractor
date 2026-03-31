use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::types::SpriteMode;
use crate::utils::labeled_file_row;
use iced::widget::{button, column, row, text};
use iced::Element;

impl App {
    pub fn view_sprite_tab(&self) -> Element<'_, Message> {
        let op_buttons: Vec<Element<Message>> = SpriteMode::ALL
            .iter()
            .map(|m| {
                let is_active = self.sprite_mode == Some(*m);
                let btn = button(text(m.to_string()).size(12))
                    .padding([6, 16])
                    .on_press(Message::SpriteModeSelected(*m));
                if is_active {
                    btn.style(style::active_chip)
                } else {
                    btn.style(style::chip)
                }
                .into()
            })
            .collect();
        column![
            row(op_buttons).spacing(6),
            labeled_file_row(
                "Input file:",
                &self.sprite_input,
                Message::SpriteInputChanged,
                Message::BrowseSpriteInput
            ),
        ]
        .spacing(12)
        .into()
    }
}
