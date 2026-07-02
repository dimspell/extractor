use iced::widget::{container, text_input};
use iced::{color, Background, Border, Color, Theme};

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

pub fn grid_cell(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        border: Border {
            color: color!(0x3d2b1f),
            width: 0.5,
            radius: 0.into(),
        },
        text_color: Some(color!(0xd7ccc8)),
        ..Default::default()
    }
}

pub fn grid_cell_dirty(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x3d2b1f))), // Darker brown for dirty
        border: Border {
            color: color!(0xdaa520), // Gold border for dirty cell
            width: 1.0,
            radius: 0.into(),
        },
        text_color: Some(color!(0xeae0c8)),
        ..Default::default()
    }
}

pub fn grid_row(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x262626))),
        ..Default::default()
    }
}

pub fn grid_row_even(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x1f1f1f))),
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
/// click-and-drag handle for resizing a column.
pub fn resize_handle(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x4a3728))),
        border: Border {
            color: color!(0x2a1f17),
            width: 0.0,
            radius: 0.into(),
        },
        ..Default::default()
    }
}
/// Data cell container — provides subtle gridline border, no own background
pub fn spreadsheet_cell(_theme: &Theme) -> container::Style {
    container::Style {
        background: None,
        border: Border {
            color: color!(0x2e2824),
            width: 0.5,
            radius: 0.into(),
        },
        ..Default::default()
    }
}
/// Row-number (#) column — darker "frozen column" look matching Excel
pub fn spreadsheet_id_cell(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x171411))),
        border: Border {
            color: color!(0x3d2b1f),
            width: 1.0,
            radius: 0.into(),
        },
        text_color: Some(color!(0x6a5e54)),
        ..Default::default()
    }
}
/// Row-number (#) column — lighter tint when the row is selected
pub fn spreadsheet_id_cell_selected(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x2a2010))),
        border: Border {
            color: color!(0xdaa520, 0.5),
            width: 1.0,
            radius: 0.into(),
        },
        text_color: Some(color!(0xdaa520)),
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
        icon: color!(0x888888),
        placeholder: color!(0x666666),
        value: color!(0xeae0c8),
        selection: color!(0xdaa520, 0.3),
    }
}
/// Text input used for in-cell editing — brighter border to signal "active edit".
pub fn spreadsheet_cell_editor(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused { .. } => color!(0xffd700),
        _ => color!(0xdaa520, 0.6),
    };
    text_input::Style {
        background: Background::Color(color!(0x2a1f18)),
        border: Border {
            color: border_color,
            width: 1.5,
            radius: 2.into(),
        },
        icon: color!(0x888888),
        placeholder: color!(0x666666),
        value: color!(0xffee58),
        selection: color!(0xdaa520, 0.4),
    }
}
/// Data cell that failed validation — red border, subtle red tint.
pub fn spreadsheet_cell_invalid(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x3a1a18))),
        border: Border {
            color: color!(0xff5252),
            width: 1.5,
            radius: 0.into(),
        },
        text_color: Some(color!(0xffcdd2)),
        ..Default::default()
    }
}
