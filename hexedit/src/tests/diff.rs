//! Integration tests for the diff view: mouse clicks/drags report which
//! side they landed on, so the inspector can follow the inspected file.

use super::*;

use iced::{Event, mouse};
use iced_test::simulator::click;

use crate::ui::view::diff::DiffView;
use crate::ui::view::diff::layout::{ADDR_COL_WIDTH, HEADER_HEIGHT, HEX_CELL_WIDTH, ROW_HEIGHT};

fn sel() -> Selection {
    Selection::single(0)
}

fn minimal_dv<'a, Message: 'a>(a: &'a [u8], b: &'a [u8], bpr: u8) -> DiffView<'a, Message> {
    static EMPTY_SET: BTreeSet<u64> = BTreeSet::new();
    static EMPTY_MAP: BTreeMap<u64, (usize, u8)> = BTreeMap::new();
    static EMPTY_ANN: BTreeMap<u64, Vec<(usize, String)>> = BTreeMap::new();
    static EMPTY_ACTIVE: BTreeSet<usize> = BTreeSet::new();
    DiffView::new(
        a,
        b,
        bpr,
        sel(),
        &EMPTY_SET,
        &EMPTY_MAP,
        &EMPTY_SET,
        0,
        None,
        &[],
        &EMPTY_ANN,
        &EMPTY_ACTIVE,
        BTreeSet::new(),
        ParagraphCache::default(),
        crate::coloring::ColorScheme::Monochrome,
        false,
        &crate::ui::theme::DARK_THEME,
    )
}

fn baseline_x(byte_col: usize) -> f32 {
    crate::ui::view::diff::layout::baseline_hex_start(ADDR_COL_WIDTH)
        + (byte_col as f32) * HEX_CELL_WIDTH
        + HEX_CELL_WIDTH / 2.0
}

fn comparison_x(byte_col: usize) -> f32 {
    crate::ui::view::diff::layout::comparison_hex_start(ADDR_COL_WIDTH, 16)
        + (byte_col as f32) * HEX_CELL_WIDTH
        + HEX_CELL_WIDTH / 2.0
}

fn row_0_y() -> f32 {
    HEADER_HEIGHT + ROW_HEIGHT / 2.0
}

#[test]
fn test_diff_click_on_comparison_side_publishes_side() {
    let mut ui = simulator::<HexEditorMessage, iced::Theme, iced::Renderer>(
        minimal_dv::<HexEditorMessage>(&[0u8; 32], &[0u8; 32], 16).on_select_at(
            |addr, is_baseline| HexEditorMessage::DiffAddrSelected { addr, is_baseline },
        ),
    );
    ui.point_at((comparison_x(0), row_0_y()));
    let _ = ui.simulate(click());
    let messages: Vec<HexEditorMessage> = ui.into_messages().collect();
    assert!(
        messages.iter().any(|m| matches!(
            m,
            HexEditorMessage::DiffAddrSelected {
                addr: 0,
                is_baseline: false
            }
        )),
        "click on the comparison side must publish is_baseline=false, got {messages:?}"
    );
}

#[test]
fn test_diff_click_on_baseline_side_publishes_side() {
    let mut ui = simulator::<HexEditorMessage, iced::Theme, iced::Renderer>(
        minimal_dv::<HexEditorMessage>(&[0u8; 32], &[0u8; 32], 16).on_select_at(
            |addr, is_baseline| HexEditorMessage::DiffAddrSelected { addr, is_baseline },
        ),
    );
    ui.point_at((baseline_x(0), row_0_y()));
    let _ = ui.simulate(click());
    let messages: Vec<HexEditorMessage> = ui.into_messages().collect();
    assert!(
        messages.iter().any(|m| matches!(
            m,
            HexEditorMessage::DiffAddrSelected {
                addr: 0,
                is_baseline: true
            }
        )),
        "click on the baseline side must publish is_baseline=true, got {messages:?}"
    );
}

#[test]
fn test_diff_drag_on_comparison_side_publishes_side() {
    // Press on the comparison side, drag one cell further, release:
    // the extend message must carry the comparison side.
    let mut ui = simulator::<HexEditorMessage, iced::Theme, iced::Renderer>(
        minimal_dv::<HexEditorMessage>(&[0u8; 32], &[0u8; 32], 16)
            .on_extend_to(|addr, is_baseline| HexEditorMessage::DiffExtendTo { addr, is_baseline }),
    );
    ui.point_at((comparison_x(0), row_0_y()));
    let _ = ui.simulate([Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    ))]);
    ui.point_at((comparison_x(1), row_0_y()));
    let _ = ui.simulate([Event::Mouse(mouse::Event::CursorMoved {
        position: iced::Point::new(comparison_x(1), row_0_y()),
    })]);
    let _ = ui.simulate([Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Left,
    ))]);
    let messages: Vec<HexEditorMessage> = ui.into_messages().collect();
    assert!(
        messages.iter().any(|m| matches!(
            m,
            HexEditorMessage::DiffExtendTo {
                addr: 1,
                is_baseline: false
            }
        )),
        "drag on the comparison side must publish DiffExtendTo with is_baseline=false, got {messages:?}"
    );
}

#[test]
fn test_diff_text_colors_contrast_in_both_themes() {
    use crate::coloring::contrast_ratio;
    use crate::ui::theme::{DARK_THEME, LIGHT_THEME};
    use crate::ui::view::diff::diff_text_color;

    for theme in [&DARK_THEME, &LIGHT_THEME] {
        let bg = theme.diff_bg;
        let (dr, dg, db) = (bg.r, bg.g, bg.b);
        let baseline_bg = iced::Color::from_rgb(
            (dr + 0.25).min(1.0),
            (dg * 0.4).max(0.0),
            (db * 0.3).max(0.0),
        );
        let comparison_bg = iced::Color::from_rgb(
            (dr * 0.4).max(0.0),
            (dg + 0.25).min(1.0),
            (db * 0.4).max(0.0),
        );
        for (cell_bg, baseline) in [(baseline_bg, true), (comparison_bg, false)] {
            let fg = diff_text_color(theme, cell_bg, baseline);
            let cr = contrast_ratio(&fg, &cell_bg);
            assert!(
                cr >= 4.0,
                "{} theme {} text contrast {cr:.2} is too low",
                if std::ptr::eq(theme, &DARK_THEME) {
                    "dark"
                } else {
                    "light"
                },
                if baseline { "baseline" } else { "comparison" }
            );
        }
    }
}
