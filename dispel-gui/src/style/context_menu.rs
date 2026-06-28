use iced::widget::container;
use iced::{color, Background, Border, Color, Shadow, Theme, Vector};


pub fn context_menu(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x3d2b1f))), // Dark leather/brown
        border: Border {
            color: color!(0x8d6e63), // Light brown border
            width: 1.0,
            radius: 4.into(),
        },
        shadow: Shadow {
            color: color!(0x000000, 0.8),
            offset: Vector::new(2.0, 2.0),
            blur_radius: 8.0,
        },
        text_color: None,
        snap: false,
    }
}

pub fn menu_separator(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x4a3a2e))),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.into(),
        },
        text_color: None,
        ..Default::default()
    }
}
