use iced::widget::{button, container, progress_bar, text, text_input};
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

// ─── Text styles ────────────────────────────────────────────────────────────

pub fn subtle_text(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(color!(0x8d6e63)), // Muted brown
    }
}

pub fn section_header(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(color!(0xeae0c8)), // Light tan for headers
    }
}

pub fn primary_text(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(color!(0x4CAF50)), // Green for primary/highlighted text
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

// ─── Progress bar styles ────────────────────────────────────────────────────

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

// ─── Progress Bar Theme Extensions ─────────────────────────────────────────

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

pub fn progress_text_style(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(color!(0xeae0c8)), // Light tan text for contrast
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

// ─── Pane Grid styles ───────────────────────────────────────────────────────

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

pub fn pane_title_focused(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(color!(0xffd700)),
    }
}

pub fn pane_title_unfocused(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(color!(0x888888)),
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

// ─── Spreadsheet styles ─────────────────────────────────────────────────────

// ─── Spreadsheet styles ──────────────────────────────────────────────────────

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

/// Thin vertical divider between column headers that doubles as the
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

/// Data row button — zebra striping, hover highlight, selected accent.
///
/// Supports two orthogonal visual states in addition to zebra striping:
///   * `is_selected` — the row is part of the current selection (brown + gold).
///   * `is_highlighted` — the row matches the current filter query in
///     `GlobalFilterMode::Highlight` (gold tint).
///   * `is_current_highlight` — this is the highlight the user is currently
///     navigating to with `Ctrl+G` / `Ctrl+Shift+G` (brighter gold, thicker
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

// ─── Status bar — VIM-style editing mode indicator ──────────────────────────

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

/// Text inside a NORMAL mode chip.
pub fn normal_mode_text(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(color!(0xd2b48c)),
    }
}

/// Text inside an EDIT mode chip.
pub fn edit_mode_text(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(color!(0xffd700)),
    }
}

// ─── Filter bar — mode toggle, nav, clear ───────────────────────────────────

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

/// Text style used for filter-bar status (e.g. "12 of 350 rows" / "7 highlighted").
pub fn filter_status_text(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(color!(0xdaa520)),
    }
}

/// Small subtle keyboard-shortcut hint (e.g. "Ctrl+F", "Ctrl+G").
pub fn shortcut_hint(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(color!(0x6e5a50)),
    }
}

// ─── Context Menu styles ──────────────────────────────────────────────────────

pub fn context_menu(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x3d2b1f))), // Dark leather/brown
        border: Border {
            color: color!(0x8d6e63), // Light brown border
            width: 1.0,
            radius: 4.into(),
        },
        shadow: Shadow {
            color: color!(0x000000, 0.8),
            offset: Vector::new(2.0, 2.0),
            blur_radius: 8.0,
        },
        text_color: None,
        snap: false,
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

// ── Sprite viewer playback controls ──────────────────────────────────────────

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
