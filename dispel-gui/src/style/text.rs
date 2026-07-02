use iced::widget::text;
use iced::{color, Theme};

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

pub fn progress_text_style(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(color!(0xeae0c8)), // Light tan text for contrast
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

pub fn menu_disabled_text(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(color!(0x706860)),
    }
}
