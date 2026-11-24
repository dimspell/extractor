use iced::widget::text;
use iced::{Theme, color};

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
