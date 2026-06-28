use iced::widget::progress_bar;
use iced::{color, Background, Border, Theme};


pub fn loading_progress_bar(_theme: &Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: Background::Color(color!(0x3d2b1f)),
        bar: Background::Color(color!(0xdaa520)), // Gold bar
        border: Border {
            color: color!(0x5d4037),
            width: 1.0,
            radius: 2.into(),
        },
    }
}

pub fn primary_progress_bar(_theme: &Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: Background::Color(color!(0x3d2b1f)), // Dark brown background
        bar: Background::Color(color!(0xdaa520)),        // Gold progress bar
        border: Border {
            color: color!(0x5d4037),
            width: 1.0,
            radius: 2.into(),
        },
    }
}

pub fn secondary_progress_bar(_theme: &Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: Background::Color(color!(0x3d2b1f)), // Dark brown background
        bar: Background::Color(color!(0xff8c00)),        // Orange warning color
        border: Border {
            color: color!(0x5d4037),
            width: 1.0,
            radius: 2.into(),
        },
    }
}
