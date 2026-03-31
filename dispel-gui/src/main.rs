use iced::color;
use iced::Theme;

pub mod app;
pub mod chest_editor;
pub mod db;
pub mod db_viewer_state;
pub mod message;
pub mod style;
pub mod types;
pub mod utils;
pub mod view;

use crate::app::App;

pub fn main() -> iced::Result {
    iced::application("Dispel Extractor", App::update, App::view)
        .theme(|_| {
            Theme::custom(
                "Dispel Dark".into(),
                iced::theme::Palette {
                    background: color!(0x1a1a2e),
                    text: color!(0xe0e0e0),
                    primary: color!(0x6c63ff),
                    success: color!(0x2ecc71),
                    danger: color!(0xe74c3c),
                },
            )
        })
        .window_size((1100.0, 800.0))
        .run_with(App::new)
}
