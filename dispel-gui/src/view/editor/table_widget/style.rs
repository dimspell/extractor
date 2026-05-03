use super::types::RowFlags;
use iced::{color, Color};

pub(crate) fn cell_text_color(flags: RowFlags) -> Color {
    if flags.current_highlight {
        color!(0xffffff)
    } else if flags.highlighted {
        color!(0xfff2c0)
    } else if flags.selected {
        color!(0xffd700)
    } else {
        color!(0xd4c5a9)
    }
}

pub(crate) fn id_text_color(flags: RowFlags) -> Color {
    if flags.selected || flags.current_highlight {
        color!(0xffd700)
    } else {
        color!(0x6a5e54)
    }
}

pub(crate) fn id_cell_bg(flags: RowFlags) -> Color {
    if flags.selected || flags.current_highlight {
        color!(0x2a2218)
    } else {
        color!(0x171411)
    }
}

pub(crate) fn row_bg(visible_idx: usize, flags: RowFlags, hovered: bool) -> Color {
    if flags.current_highlight {
        color!(0x7a6a2a)
    } else if flags.highlighted {
        color!(0x5a4e1a)
    } else if flags.selected {
        color!(0x3a2e1a)
    } else if hovered {
        color!(0x2d2820)
    } else if visible_idx.is_multiple_of(2) {
        color!(0x1e1b17)
    } else {
        color!(0x232019)
    }
}

pub(crate) fn row_border(flags: RowFlags) -> Option<(Color, f32)> {
    if flags.current_highlight {
        Some((color!(0xffd700, 0.85), 2.0))
    } else if flags.highlighted {
        Some((color!(0xdaa520, 0.7), 1.0))
    } else if flags.selected {
        Some((color!(0xdaa520, 0.5), 1.0))
    } else {
        None
    }
}
