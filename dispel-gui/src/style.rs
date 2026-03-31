use iced::widget::{button, container, text};
use iced::{color, Background, Border, Color, Shadow, Theme, Vector};

// ─── Container styles ───────────────────────────────────────────────────────

pub fn root_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x1a1a2e))),
        ..Default::default()
    }
}

pub fn sidebar_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x16213e))),
        border: Border {
            color: color!(0x2a2a4a),
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
        background: Some(Background::Color(color!(0x0f0f23))),
        border: Border {
            color: color!(0x2a2a4a),
            width: 1.0,
            radius: 0.into(),
        },
        ..Default::default()
    }
}

pub fn info_card(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x1e1e3a))),
        border: Border {
            color: color!(0x3a3a5c),
            width: 1.0,
            radius: 8.into(),
        },
        ..Default::default()
    }
}

// ─── Button styles ──────────────────────────────────────────────────────────

pub fn tab_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: color!(0x8888aa),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 6.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x252548))),
            text_color: color!(0xccccee),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(color!(0x2a2a50))),
            text_color: color!(0xeeeeff),
            ..base
        },
        _ => base,
    }
}

pub fn active_tab_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(color!(0x6c63ff, 0.15))),
        text_color: color!(0x9b93ff),
        border: Border {
            color: color!(0x6c63ff),
            width: 0.0,
            radius: 6.into(),
        },
        shadow: Shadow {
            color: color!(0x6c63ff, 0.1),
            offset: Vector::ZERO,
            blur_radius: 8.0,
        },
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x6c63ff, 0.25))),
            text_color: color!(0xb0aaff),
            ..base
        },
        _ => base,
    }
}

pub fn run_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(color!(0x6c63ff))),
        text_color: Color::WHITE,
        border: Border {
            color: color!(0x7c73ff),
            width: 0.0,
            radius: 8.into(),
        },
        shadow: Shadow {
            color: color!(0x6c63ff, 0.4),
            offset: Vector::new(0.0, 3.0),
            blur_radius: 12.0,
        },
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x7c73ff))),
            shadow: Shadow {
                color: color!(0x6c63ff, 0.6),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 16.0,
            },
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(color!(0x5c53ef))),
            shadow: Shadow {
                color: color!(0x6c63ff, 0.2),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 4.0,
            },
            ..base
        },
        _ => base,
    }
}

pub fn run_button_disabled(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(color!(0x3a3a5c))),
        text_color: color!(0x666688),
        border: Border {
            color: color!(0x3a3a5c),
            width: 0.0,
            radius: 8.into(),
        },
        ..Default::default()
    }
}

pub fn chip(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(color!(0x252548))),
        text_color: color!(0x9999bb),
        border: Border {
            color: color!(0x3a3a5c),
            width: 1.0,
            radius: 16.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x2e2e55))),
            text_color: color!(0xccccee),
            border: Border {
                color: color!(0x5555aa),
                width: 1.0,
                radius: 16.into(),
            },
            ..base
        },
        _ => base,
    }
}

pub fn active_chip(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(color!(0x6c63ff, 0.2))),
        text_color: color!(0xb0aaff),
        border: Border {
            color: color!(0x6c63ff),
            width: 1.0,
            radius: 16.into(),
        },
        shadow: Shadow {
            color: color!(0x6c63ff, 0.15),
            offset: Vector::ZERO,
            blur_radius: 6.0,
        },
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x6c63ff, 0.3))),
            ..base
        },
        _ => base,
    }
}

pub fn browse_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(color!(0x2a2a50))),
        text_color: color!(0xaaaacc),
        border: Border {
            color: color!(0x3a3a5c),
            width: 1.0,
            radius: 4.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x353560))),
            text_color: color!(0xddddee),
            ..base
        },
        _ => base,
    }
}

// ─── Text styles ────────────────────────────────────────────────────────────

pub fn subtle_text(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(color!(0x8888aa)),
    }
}

// ─── Spreadsheet / Database Viewer styles ───────────────────────────────────

pub fn grid_header_cell(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x252548))),
        border: Border {
            color: color!(0x3a3a5c),
            width: 1.0,
            radius: 0.into(),
        },
        ..Default::default()
    }
}

pub fn grid_cell(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x1a1a2e))),
        border: Border {
            color: color!(0x2a2a4a),
            width: 0.5,
            radius: 0.into(),
        },
        ..Default::default()
    }
}

pub fn grid_cell_dirty(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x1a2a4e))),
        border: Border {
            color: color!(0x4488cc),
            width: 1.0,
            radius: 0.into(),
        },
        ..Default::default()
    }
}

pub fn grid_cell_even(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x181830))),
        border: Border {
            color: color!(0x2a2a4a),
            width: 0.5,
            radius: 0.into(),
        },
        ..Default::default()
    }
}

pub fn toolbar_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x16213e))),
        border: Border {
            color: color!(0x2a2a4a),
            width: 0.0,
            radius: 0.into(),
        },
        ..Default::default()
    }
}

pub fn sql_editor_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x0f0f23))),
        border: Border {
            color: color!(0x3a3a5c),
            width: 1.0,
            radius: 6.into(),
        },
        ..Default::default()
    }
}

pub fn sort_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: color!(0xaaaacc),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            text_color: color!(0xeeeeff),
            background: Some(Background::Color(color!(0x2e2e55))),
            ..base
        },
        _ => base,
    }
}

pub fn commit_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(color!(0x2ecc71))),
        text_color: Color::WHITE,
        border: Border {
            color: color!(0x27ae60),
            width: 0.0,
            radius: 6.into(),
        },
        shadow: Shadow {
            color: color!(0x2ecc71, 0.3),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x3ddc84))),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(color!(0x27ae60))),
            ..base
        },
        _ => base,
    }
}

pub fn status_bar(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x12122a))),
        border: Border {
            color: color!(0x2a2a4a),
            width: 1.0,
            radius: 0.into(),
        },
        ..Default::default()
    }
}
