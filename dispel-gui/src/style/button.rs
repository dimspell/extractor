use iced::widget::button;
use iced::{color, Background, Border, Color, Shadow, Theme, Vector};


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
        text_color: color!(0xffd700),                               // Gold
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
        snap: false,
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
        snap: false,
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
        snap: false,
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
        text_color: color!(0xbdbdbd),                          // Silver
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
        snap: false,
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

pub fn selected_button(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(color!(0x5d4037))),
        text_color: color!(0xffd700),
        border: Border {
            color: color!(0xdaa520),
            width: 1.0,
            radius: 4.into(),
        },
        shadow: Shadow {
            color: color!(0xdaa520, 0.2),
            offset: Vector::ZERO,
            blur_radius: 4.0,
        },
        snap: false,
    }
}

pub fn selected_row_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(color!(0x8b5a2b, 0.25))),
        text_color: color!(0xd2b48c),
        border: Border {
            color: color!(0xdaa520, 0.5),
            width: 1.0,
            radius: 0.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x8b5a2b, 0.35))),
            ..base
        },
        _ => base,
    }
}

pub fn normal_row_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: color!(0xbcaaa4),
        border: Border {
            color: color!(0x3d2b1f, 0.2),
            width: 1.0,
            radius: 0.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x3d2b1f, 0.3))),
            ..base
        },
        _ => base,
    }
}

pub fn pane_header_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: color!(0xcccccc),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 2.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x5d4037))),
            text_color: color!(0xffd700),
            ..base
        },
        _ => base,
    }
}
/// Header cell button — transparent base, column separator via border, hover highlight
pub fn spreadsheet_header_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: None,
        text_color: color!(0xb8a898),
        border: Border {
            color: color!(0x3d2b1f),
            width: 1.0,
            radius: 0.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x2d2218))),
            text_color: color!(0xeae0c8),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(color!(0x3d2b1f))),
            text_color: color!(0xffd700),
            ..base
        },
        _ => base,
    }
}
///     border).
pub fn spreadsheet_row(
    is_selected: bool,
    row_idx: usize,
    is_highlighted: bool,
    is_current_highlight: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let (bg, tc) = if is_current_highlight {
            // "Find next match" target — brightest, most prominent
            (color!(0x7a6a2a), color!(0xffffff))
        } else if is_highlighted {
            // Other rows that match in highlight mode
            (color!(0x5a4e1a), color!(0xfff2c0))
        } else if is_selected {
            (color!(0x3a2e1a), color!(0xffd700))
        } else if row_idx.is_multiple_of(2) {
            (color!(0x1e1b17), color!(0xd4c5a9))
        } else {
            (color!(0x232019), color!(0xd4c5a9))
        };

        let final_bg = match status {
            button::Status::Hovered if !is_selected && !is_highlighted && !is_current_highlight => {
                color!(0x2d2820)
            }
            _ => bg,
        };

        let border_color = if is_current_highlight {
            color!(0xffd700, 0.85)
        } else if is_highlighted {
            color!(0xdaa520, 0.7)
        } else if is_selected {
            color!(0xdaa520, 0.5)
        } else {
            color!(0x2a2520)
        };

        let border_width = if is_current_highlight {
            2.0
        } else if is_selected || is_highlighted {
            1.0
        } else {
            0.5
        };

        button::Style {
            background: Some(Background::Color(final_bg)),
            text_color: tc,
            border: Border {
                color: border_color,
                width: border_width,
                radius: 0.into(),
            },
            ..Default::default()
        }
    }
}
/// Data cell button — fully transparent, no decoration; row background and selection show through
pub fn spreadsheet_cell_btn(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: None,
        text_color: color!(0xd4c5a9),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.into(),
        },
        ..Default::default()
    }
}
/// Highlighted (selected) filter mode button — gold accent.
pub fn filter_mode_active(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(color!(0x8b5a2b))),
        text_color: color!(0xffd700),
        border: Border {
            color: color!(0xdaa520),
            width: 1.0,
            radius: 3.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0xa06a3b))),
            ..base
        },
        _ => base,
    }
}
/// Dim filter mode button — shown when the mode is not active.
pub fn filter_mode_inactive(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(color!(0x2a1f18))),
        text_color: color!(0xa1887f),
        border: Border {
            color: color!(0x5d4037),
            width: 1.0,
            radius: 3.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x3d2b1f))),
            text_color: color!(0xd7ccc8),
            ..base
        },
        _ => base,
    }
}
/// Small circular "×" button used to clear the filter input.
pub fn filter_clear_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(color!(0x5d4037))),
        text_color: color!(0xeae0c8),
        border: Border {
            color: color!(0x8b5a2b),
            width: 1.0,
            radius: 12.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x8b2f2f))),
            text_color: color!(0xffffff),
            ..base
        },
        _ => base,
    }
}
/// Prev / Next highlight navigation button (shown in Highlight mode).
pub fn nav_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(color!(0x3d2b1f))),
        text_color: color!(0xffd700),
        border: Border {
            color: color!(0x8b5a2b),
            width: 1.0,
            radius: 3.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x5d4037))),
            text_color: color!(0xffee58),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(color!(0x2a1f18))),
            ..base
        },
        _ => base,
    }
}
/// File tree directory row — caret + name, no border, hover highlight
pub fn tree_dir_row(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: None,
        text_color: color!(0xd7ccc8), // Slightly brighter for dirs
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x2a1f14))),
            text_color: color!(0xeae0c8),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(color!(0x3d2b1f))),
            text_color: color!(0xffd700),
            ..base
        },
        _ => base,
    }
}
/// File tree file row — icon + name, no border, subtle hover
pub fn tree_file_row(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: None,
        text_color: color!(0xa1887f), // Subdued for files
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x261912))),
            text_color: color!(0xd7ccc8),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(color!(0x3d2b1f))),
            text_color: color!(0xeae0c8),
            ..base
        },
        _ => base,
    }
}
/// File tree menu button (⋮) — minimal, subtle
pub fn tree_menu_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: None,
        text_color: color!(0x8d6e63),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x3d2b1f))),
            text_color: color!(0xd7ccc8),
            ..base
        },
        _ => base,
    }
}

pub fn menu_disabled_item(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: color!(0x706860),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 4.into(),
        },
        ..Default::default()
    }
}

pub fn menu_item(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: color!(0xeae0c8), // Light tan text
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 2.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x5d4037))), // Highlighted leather
            text_color: color!(0xffd700),                          // Gold text on hover
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(color!(0x4e342e))), // Darker highlight
            text_color: color!(0xffee58),                          // Brighter gold when pressed
            ..base
        },
        _ => base,
    }
}
/// Playback transport buttons (play, pause, step, loop).
pub fn playback_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(color!(0x2a1f18))),
        text_color: color!(0xd7ccc8),
        border: Border {
            color: color!(0x5d4037),
            width: 1.0,
            radius: 4.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x3d2b1f))),
            text_color: color!(0xffd700),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(color!(0x4e342e))),
            text_color: color!(0xffee58),
            ..base
        },
        _ => base,
    }
}
/// Playback button that is currently "active" (e.g. loop is enabled).
pub fn playback_button_active(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(color!(0x5d4037))),
        text_color: color!(0xffd700),
        border: Border {
            color: color!(0xa1887f),
            width: 1.0,
            radius: 4.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x6d4c41))),
            text_color: color!(0xffee58),
            ..base
        },
        _ => base,
    }
}
/// Export / action button with a gold accent.
pub fn export_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(color!(0x3d2b1f))),
        text_color: color!(0xffd700),
        border: Border {
            color: color!(0x8d6e63),
            width: 1.0,
            radius: 4.into(),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x5d4037))),
            text_color: color!(0xffee58),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(color!(0x4e342e))),
            ..base
        },
        _ => base,
    }
}
