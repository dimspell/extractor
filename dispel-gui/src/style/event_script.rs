use iced::widget::{button, container};
use iced::{color, Background, Border, Color, Theme};

pub fn badge_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x3d2b1f))),
        border: Border {
            color: color!(0x5d4037),
            width: 1.0,
            radius: 3.into(),
        },
        text_color: Some(color!(0xd2b48c)),
        ..Default::default()
    }
}

pub fn badge_if_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0xff8f00))),
        border: Border {
            color: color!(0xffa000),
            width: 1.0,
            radius: 3.into(),
        },
        text_color: Some(color!(0x1a1510)),
        ..Default::default()
    }
}

pub fn badge_else_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0xe65100))),
        border: Border {
            color: color!(0xef6c00),
            width: 1.0,
            radius: 3.into(),
        },
        text_color: Some(color!(0xffffff)),
        ..Default::default()
    }
}

pub fn badge_ret_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x00897b))),
        border: Border {
            color: color!(0x26a69a),
            width: 1.0,
            radius: 3.into(),
        },
        text_color: Some(color!(0xffffff)),
        ..Default::default()
    }
}

pub fn badge_func_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x1565c0))),
        border: Border {
            color: color!(0x1976d2),
            width: 1.0,
            radius: 3.into(),
        },
        text_color: Some(color!(0xffffff)),
        ..Default::default()
    }
}

pub fn badge_text_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x546e7a))),
        border: Border {
            color: color!(0x78909c),
            width: 1.0,
            radius: 3.into(),
        },
        text_color: Some(color!(0xeceff1)),
        ..Default::default()
    }
}

pub fn move_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: color!(0x8d6e63),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 2.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            text_color: color!(0xd7ccc8),
            background: Some(Background::Color(color!(0x3d2b1f, 0.5))),
            ..base
        },
        _ => base,
    }
}

pub fn fold_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: color!(0xa1887f),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 2.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            text_color: color!(0xd7ccc8),
            background: Some(Background::Color(color!(0x3d2b1f, 0.5))),
            ..base
        },
        _ => base,
    }
}
