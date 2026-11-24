use iced::widget::{container, text_input};
use iced::{Background, Border, Theme, color};

pub fn grid_header_cell(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x3e2723))), // Dark wood header
        border: Border {
            color: color!(0x5d4037),
            width: 1.0,
            radius: 0.into(),
        },
        ..Default::default()
    }
}

/// Header row container — dark background, bottom separator via border
pub fn spreadsheet_header(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x1c1813))),
        border: Border {
            color: color!(0x4a3728),
            width: 1.0,
            radius: 0.into(),
        },
        text_color: Some(color!(0xb8a898)),
        ..Default::default()
    }
}

pub fn spreadsheet_filter_input(_theme: &Theme, _status: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: Background::Color(color!(0x1a1510)),
        border: Border {
            color: color!(0x5d4037),
            width: 1.0,
            radius: 4.into(),
        },
        placeholder: color!(0x666666),
        value: color!(0xeae0c8),
        selection: color!(0xdaa520, 0.3),
    }
}
