use iced::widget::container;
use iced::{color, Background, Border, Color, Theme};


pub fn pane_focused(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x2a2a2a))),
        border: Border {
            color: color!(0xdaa520, 0.6),
            width: 2.0,
            radius: 4.into(),
        },
        ..Default::default()
    }
}

pub fn pane_unfocused(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x222222))),
        border: Border {
            color: color!(0x3d2b1f, 0.4),
            width: 1.0,
            radius: 4.into(),
        },
        ..Default::default()
    }
}

pub fn pane_header_focused(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x3d2b1f))),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.into(),
        },
        ..Default::default()
    }
}

pub fn pane_header_unfocused(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x2d2d2d))),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.into(),
        },
        ..Default::default()
    }
}
