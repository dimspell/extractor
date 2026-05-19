use iced::widget::{button, container, text};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

/// Context menu container — dark leather/brown panel, light brown border.
pub fn context_menu(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.24, 0.17, 0.12))),
        border: Border {
            color: Color::from_rgb(0.55, 0.43, 0.39),
            width: 1.0,
            radius: 4.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.8),
            offset: Vector::new(2.0, 2.0),
            blur_radius: 8.0,
        },
        text_color: None,
        ..Default::default()
    }
}

/// Context menu item — transparent bg, light tan text, gold highlight on hover.
pub fn menu_item(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: Color::from_rgb(0.92, 0.88, 0.78),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 2.0.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.36, 0.25, 0.22))),
            text_color: Color::from_rgb(1.0, 0.84, 0.0),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.31, 0.20, 0.18))),
            text_color: Color::from_rgb(1.0, 0.93, 0.35),
            ..base
        },
        _ => base,
    }
}

/// Context menu separator — thin leather-colored line.
pub fn menu_separator(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.29, 0.23, 0.18))),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        text_color: None,
        ..Default::default()
    }
}

/// Disabled context menu item button.
pub fn menu_disabled_item(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: Color::from_rgb(0.44, 0.41, 0.38),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

/// Disabled context menu item text color.
pub fn menu_disabled_text(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(Color::from_rgb(0.44, 0.41, 0.38)),
    }
}
