use iced::color;
use iced::Theme;

pub mod app;
pub mod auto_save;
pub mod components;
pub mod editors;
pub mod indexation;
pub mod message;
#[allow(clippy::module_inception)]
pub mod state;
pub mod style;
pub mod update;
pub mod view;
pub mod workspace;

use crate::app::App;

pub fn main() -> iced::Result {
    // Initialize logging
    env_logger::init();

    iced::application(App::new, App::update, App::view)
        .theme(|_: &App| {
            Theme::custom(
                "Medieval",
                iced::theme::Palette {
                    background: color!(0x2a2a2a),
                    text: color!(0xeae0c8),
                    primary: color!(0x8b5a2b),
                    success: color!(0x2d5a27),
                    danger: color!(0x800000),
                    warning: color!(0x8b8b00),
                },
            )
        })
        .title("Dispel Extractor")
        .subscription(App::subscription)
        .window_size((1100.0, 800.0))
        .run()
}
