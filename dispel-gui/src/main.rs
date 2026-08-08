// Prevent a cmd.exe console window from appearing alongside the GUI on Windows.
#![windows_subsystem = "windows"]

use iced::Theme;
use iced::color;

pub mod app;
pub mod components;
pub mod dispatch_table;
pub mod editor_registry;
pub mod editors;
pub mod indexation;
pub mod message;
pub mod subscriptions;

pub mod platform;
pub mod state;
pub mod style;
pub mod update;
pub mod view;
pub mod workspace;

#[cfg(test)]
mod recording_tests;

#[cfg(test)]
mod tests;

use crate::app::App;

pub fn main() -> iced::Result {
    // Initialize logging
    env_logger::init();

    iced::application(App::new, App::update, App::view)
        .font(lucide_icons::LUCIDE_FONT_BYTES)
        .theme(|_: &App| {
            Theme::custom(
                "Medieval",
                iced::theme::palette::Seed {
                    background: color!(0x2a2a2a),
                    text: color!(0xeae0c8),
                    primary: color!(0x8b5a2b),
                    success: color!(0x2d5a27),
                    danger: color!(0x800000),
                    warning: color!(0x8b8b00),
                },
            )
        })
        .title(|app: &App| App::set_title(app))
        .subscription(|app: &App| crate::subscriptions::subscription(app))
        .window_size((1100.0, 800.0))
        .run()
}
