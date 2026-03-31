use iced::widget::{button, container, text};
use iced::{color, Background, Border, Color, Shadow, Theme, Vector};

// ─── Container styles ───────────────────────────────────────────────────────

pub fn root_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x2a2a2a))),
        ..Default::default()
    }
}

pub fn sidebar_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x1a1510))), // Deep Dark Wood/Leather
        border: Border {
            color: color!(0x3d2b1f),
            width: 0.0,
            radius: 0.into(),
        },
        shadow: Shadow {
            color: Color::BLACK,
            offset: Vector::new(2.0, 0.0),
            blur_radius: 8.0,
        },
        text_color: None,
    }
}

pub fn log_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x121212))), // Deep stone
        border: Border {
            color: color!(0x424242),
            width: 1.0,
            radius: 0.into(),
        },
        ..Default::default()
    }
}

pub fn info_card(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x2d1f1b))), // Dark leather card
        border: Border {
            color: color!(0x5d4037),
            width: 1.0,
            radius: 4.into(), // Less round, more rustic
        },
        ..Default::default()
    }
}

// ─── Button styles ──────────────────────────────────────────────────────────

pub fn tab_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: color!(0xa1887f), // Tan/Light Brown
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 4.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x3d2b1f))),
            text_color: color!(0xd7ccc8), // Light Tan
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(color!(0x2d1f1b))),
            text_color: color!(0xeae0c8),
            ..base
        },
        _ => base,
    }
}

pub fn active_tab_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(color!(0x5d4037, 0.4))), // Highlighted leather
        text_color: color!(0xffd700), // Gold
        border: Border {
            color: color!(0xdaa520), // Brass/Gold
            width: 1.0,
            radius: 4.into(),
        },
        shadow: Shadow {
            color: color!(0x5d4037, 0.2),
            offset: Vector::ZERO,
            blur_radius: 4.0,
        },
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x5d4037, 0.6))),
            text_color: color!(0xffee58), // Brighter Gold
            ..base
        },
        _ => base,
    }
}

pub fn run_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(color!(0x8b5a2b))), // Deep leather brown
        text_color: color!(0xeae0c8),
        border: Border {
            color: color!(0x5d4037),
            width: 1.0,
            radius: 4.into(),
        },
        shadow: Shadow {
            color: color!(0x000000, 0.3),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 4.0,
        },
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0xa06a3b))),
            shadow: Shadow {
                color: color!(0x000000, 0.4),
                offset: Vector::new(0.0, 3.0),
                blur_radius: 6.0,
            },
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(color!(0x6d4c41))),
            shadow: Shadow {
                color: color!(0x000000, 0.2),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
            ..base
        },
        _ => base,
    }
}

pub fn run_button_disabled(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(color!(0x3d2b1f))),
        text_color: color!(0x757575), // Silver gray text
        border: Border {
            color: color!(0x2d1f1b),
            width: 1.0,
            radius: 4.into(),
        },
        ..Default::default()
    }
}

pub fn chip(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(color!(0x3e2723))),
        text_color: color!(0xbcaaa4), // Tan text
        border: Border {
            color: color!(0x5d4037),
            width: 1.0,
            radius: 4.into(), // Medieval chips aren't so pills
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x4e342e))),
            text_color: color!(0xd7ccc8),
            border: Border {
                color: color!(0x8d6e63),
                width: 1.0,
                radius: 4.into(),
            },
            ..base
        },
        _ => base,
    }
}

pub fn active_chip(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(color!(0x8b5a2b, 0.2))),
        text_color: color!(0xd2b48c), // Tan
        border: Border {
            color: color!(0xdaa520), // Gold
            width: 1.0,
            radius: 4.into(),
        },
        shadow: Shadow {
            color: color!(0xdaa520, 0.1),
            offset: Vector::ZERO,
            blur_radius: 4.0,
        },
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x8b5a2b, 0.3))),
            ..base
        },
        _ => base,
    }
}

pub fn browse_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(color!(0x424242))), // Silver dark button
        text_color: color!(0xbdbdbd), // Silver
        border: Border {
            color: color!(0x616161),
            width: 1.0,
            radius: 2.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x757575))),
            text_color: Color::WHITE,
            ..base
        },
        _ => base,
    }
}

// ─── Text styles ────────────────────────────────────────────────────────────

pub fn subtle_text(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(color!(0x8d6e63)), // Muted brown
    }
}

// ─── Spreadsheet / Database Viewer styles ───────────────────────────────────

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

pub fn toolbar_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x1a1510))),
        border: Border {
            color: color!(0x3d2b1f),
            width: 0.0,
            radius: 0.into(),
        },
        ..Default::default()
    }
}

pub fn sql_editor_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x121212))),
        border: Border {
            color: color!(0x5d4037),
            width: 1.0,
            radius: 4.into(),
        },
        ..Default::default()
    }
}

pub fn grid_header_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: color!(0xd7ccc8), // Light tan for dark background
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            text_color: color!(0xeae0c8),
            background: Some(Background::Color(color!(0x4e342e))),
            ..base
        },
        _ => base,
    }
}

pub fn grid_cell_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: color!(0xd7ccc8), // Light tan for dark background
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x333333))),
            text_color: color!(0xeae0c8),
            ..base
        },
        _ => base,
    }
}

pub fn commit_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(color!(0x2d5a27))), // Forest Green
        text_color: color!(0xeae0c8),
        border: Border {
            color: color!(0x1b3517),
            width: 1.0,
            radius: 4.into(),
        },
        shadow: Shadow {
            color: color!(0x2d5a27, 0.2),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 4.0,
        },
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x3d7a36))),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(color!(0x1b3517))),
            ..base
        },
        _ => base,
    }
}

pub fn status_bar(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x1a1510))),
        border: Border {
            color: color!(0x3d2b1f),
            width: 1.0,
            radius: 0.into(),
        },
        ..Default::default()
    }
}
