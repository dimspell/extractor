use crate::app::App;
use crate::message::Message;
use crate::utils::{labeled_file_row, labeled_input};
use iced::widget::column;
use iced::Element;

impl App {
    pub fn view_sound_tab(&self) -> Element<'_, Message> {
        column![
            labeled_file_row(
                "Input (.SNF):",
                &self.sound_input,
                Message::SoundInputChanged,
                Message::BrowseSoundInput
            ),
            labeled_input(
                "Output (.WAV):",
                &self.sound_output,
                Message::SoundOutputChanged
            ),
        ]
        .spacing(12)
        .into()
    }
}
