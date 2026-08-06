use iced::widget::container;
use iced::{Background, Border, Color, Shadow, Theme, Vector, color};

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
        snap: false,
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

pub fn modal_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x2a2a2a))),
        border: Border {
            color: color!(0x5d4037),
            width: 1.0,
            radius: 8.into(),
        },
        shadow: Shadow {
            color: Color::BLACK,
            offset: Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        snap: false,
        text_color: None,
    }
}

pub fn selected_row(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x8b5a2b, 0.15))),
        border: Border {
            color: color!(0xdaa520, 0.3),
            width: 1.0,
            radius: 2.into(),
        },
        ..Default::default()
    }
}

pub fn normal_row(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        border: Border {
            color: color!(0x3d2b1f, 0.3),
            width: 1.0,
            radius: 2.into(),
        },
        ..Default::default()
    }
}
/// Mode chip container (NORMAL).
pub fn normal_mode_chip(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x2a1f18))),
        border: Border {
            color: color!(0x5d4037),
            width: 1.0,
            radius: 3.into(),
        },
        text_color: Some(color!(0xbcaaa4)),
        ..Default::default()
    }
}
/// Mode chip container (EDIT) — leather brown with gold border.
pub fn edit_mode_chip(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x8b5a2b))),
        border: Border {
            color: color!(0xffd700),
            width: 1.0,
            radius: 3.into(),
        },
        text_color: Some(color!(0xffd700)),
        ..Default::default()
    }
}
/// Container for the export modal dialog.
pub fn export_dialog_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x1a1510))),
        border: Border {
            color: color!(0x5d4037),
            width: 1.0,
            radius: 6.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 20.0,
        },
        text_color: Some(color!(0xd7ccc8)),
        ..Default::default()
    }
}
/// Sidebar-style panel for the entity inspector in the map editor.
pub fn inspector_container(_theme: &Theme) -> container::Style {
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

pub fn panel_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x1e1b17))),
        border: Border {
            color: color!(0x3d2b1f),
            width: 1.0,
            radius: 4.into(),
        },
        ..Default::default()
    }
}

pub fn error_row_border(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x1a1510))),
        border: Border {
            color: color!(0xc62828),
            width: 2.0,
            radius: 4.into(),
        },
        ..Default::default()
    }
}
