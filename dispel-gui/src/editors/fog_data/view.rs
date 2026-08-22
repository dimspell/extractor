use crate::app::App;
use crate::components::utils::horizontal_rule;
use crate::editors::fog_data::message::FogDataMessage;
use crate::editors::fog_data::state::FogDataEditorState;
use crate::message::{Message, MessageExt};
use crate::style;
use gui_widgets::lucide::{LUCIDE_FONT, icon_char};
use iced::widget::{button, column, container, row, scrollable, space::Space, text, text_input};
use iced::{Alignment, Color, Element, Fill, Length};
use lucide_icons::Icon;

/// Id of the scrollable level list (used to keep the selection on screen).
pub const LEVEL_LIST_ID: &str = "fog-data-level-list";
const LEVEL_LIST_WIDTH: f32 = 76.0;

/// Shorthand for wrapping a message into the top-level `Message`.
fn fog(m: FogDataMessage) -> Message {
    Message::fog_data(m)
}

/// Warm danger tone for inline validation errors.
fn error_text(theme: &iced::Theme) -> text::Style {
    let _ = theme;
    text::Style {
        color: Some(Color::from_rgb(0.90, 0.45, 0.38)),
    }
}

pub fn view(app: &App) -> Element<'_, Message> {
    let tab_id = app
        .state
        .workspace
        .active()
        .map(|t| t.id)
        .unwrap_or(usize::MAX);

    let Some(editor) = app.state.editors.fog_editors.get(&tab_id) else {
        return container(text("Fog data not loaded").size(14))
            .width(Fill)
            .height(Fill)
            .padding(16)
            .accessible_label("Fog data editor")
            .into();
    };

    // Parse failure → dedicated error surface; never panic.
    if let Some(ref err) = editor.error {
        return container(
            column![
                text("Failed to load fogdata.dat")
                    .size(15)
                    .style(style::primary_text),
                text(err).size(12).style(style::subtle_text),
                text("The file is missing or not exactly 62,976 bytes.")
                    .size(12)
                    .style(style::subtle_text),
            ]
            .spacing(8),
        )
        .width(Fill)
        .height(Fill)
        .padding(16)
        .accessible_label("Fog data editor")
        .into();
    }

    let base = column![
        view_header(tab_id, editor),
        horizontal_rule(1),
        row![
            view_level_list(tab_id, editor),
            view_curve_and_inspector(tab_id, editor)
        ]
        .width(Fill)
        .height(Fill),
    ]
    .width(Fill)
    .height(Fill);

    // Revert confirmation overlays the editor when dirty.
    if editor.confirm_revert {
        use gui_widgets::components::modal::modal;
        let base_element: Element<'_, Message> = base.into();
        let tab = tab_id;
        modal(
            base_element,
            view_revert_confirm(tab),
            move || fog(FogDataMessage::RevertCancelled(tab)),
            0.5,
        )
    } else {
        base.into()
    }
}

// ── Header / toolbar ──────────────────────────────────────────────────────────

fn view_header(tab_id: usize, editor: &FogDataEditorState) -> Element<'_, Message> {
    let title: Element<'_, Message> = if editor.dirty {
        row![
            text("fogdata.dat").size(13).style(style::primary_text),
            text(" *").size(13).style(error_text),
        ]
        .spacing(0)
        .align_y(Alignment::Center)
        .into()
    } else {
        text("fogdata.dat")
            .size(13)
            .style(style::primary_text)
            .into()
    };

    // Level stepping.
    let level = editor.selected_level;
    let prev = button(
        text(icon_char(Icon::ChevronLeft))
            .font(LUCIDE_FONT)
            .size(12),
    )
    .on_press_maybe((level > 1).then(|| fog(FogDataMessage::LevelSelected(tab_id, level - 1))))
    .padding([3, 7])
    .style(style::playback_button);

    let level_label = text(format!("Level {level} / {}", editor.level_count()))
        .size(12)
        .style(style::section_header);

    let next = button(
        text(icon_char(Icon::ChevronRight))
            .font(LUCIDE_FONT)
            .size(12),
    )
    .on_press_maybe(
        ((level as usize) < editor.level_count())
            .then(|| fog(FogDataMessage::LevelSelected(tab_id, level + 1))),
    )
    .padding([3, 7])
    .style(style::playback_button);

    // History controls.
    let undo_btn = button(text(icon_char(Icon::Undo2)).font(LUCIDE_FONT).size(12))
        .on_press_maybe(editor.can_undo().then(|| fog(FogDataMessage::Undo(tab_id))))
        .padding([3, 7])
        .style(style::playback_button);

    let redo_btn = button(text(icon_char(Icon::Redo2)).font(LUCIDE_FONT).size(12))
        .on_press_maybe(editor.can_redo().then(|| fog(FogDataMessage::Redo(tab_id))))
        .padding([3, 7])
        .style(style::playback_button);

    // File actions.
    let revert_btn = button(text("Revert").size(12))
        .on_press(fog(FogDataMessage::Revert(tab_id)))
        .padding([4, 10])
        .style(style::chip);

    let save_label = if editor.dirty { "Save *" } else { "Save" };
    let save_btn = button(text(save_label).size(12))
        .on_press(fog(FogDataMessage::Save(tab_id)))
        .padding([4, 10])
        .style(style::export_button);

    container(
        row![
            title,
            Space::new().width(Fill),
            prev,
            level_label,
            next,
            Space::new().width(8),
            undo_btn,
            redo_btn,
            Space::new().width(4),
            revert_btn,
            save_btn,
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .padding([6, 12])
    .width(Fill)
    .into()
}

// ── Level list ────────────────────────────────────────────────────────────────

fn view_level_list(tab_id: usize, editor: &FogDataEditorState) -> Element<'_, Message> {
    let selected = editor.selected_level;

    let rows: Vec<Element<'_, Message>> = (1..=editor.level_count() as u32)
        .map(|level| {
            let label: Element<'_, Message> = text(level.to_string())
                .size(11)
                .style(if selected == level {
                    style::section_header
                } else {
                    style::subtle_text
                })
                .into();
            button(container(label).width(Fill).align_x(Alignment::End))
                .on_press(fog(FogDataMessage::LevelSelected(tab_id, level)))
                .padding([3, 10])
                .width(Fill)
                .style(if selected == level {
                    style::selected_row_button
                } else {
                    style::normal_row_button
                })
                .into()
        })
        .collect();

    container(
        scrollable(column(rows).spacing(1).padding([4, 4]))
            .id(iced::widget::Id::new(LEVEL_LIST_ID))
            .width(Fill)
            .height(Fill),
    )
    .width(Length::Fixed(LEVEL_LIST_WIDTH))
    .height(Fill)
    .into()
}

// ── Curve + inspector strip ───────────────────────────────────────────────────

fn view_curve_and_inspector(tab_id: usize, editor: &FogDataEditorState) -> Element<'_, Message> {
    let canvas = super::curve_canvas::FogCurveCanvas {
        tab_id,
        row: editor.current_row(),
        selected_pair: editor.selected_pair,
    };

    column![
        container(canvas.into_element())
            .width(Fill)
            .height(Fill)
            .padding(8),
        horizontal_rule(1),
        view_inspector(tab_id, editor),
    ]
    .width(Fill)
    .height(Fill)
    .spacing(0)
    .into()
}

fn view_inspector(tab_id: usize, editor: &FogDataEditorState) -> Element<'_, Message> {
    let pair = editor.selected_pair;
    let value = editor.selected_factor().unwrap_or(0);
    let max = dispel_core::map::fogdata::MAX_FACTOR;

    let readout = format!(
        "Level {} · Pair {} · brightness {}/{} · {:.0}% light",
        editor.selected_level,
        pair,
        value,
        max,
        value as f32 / 32.0 * 100.0,
    );

    let value_field = text_input("", &editor.value_input)
        .width(Length::Fixed(56.0))
        .on_input(move |s| fog(FogDataMessage::ValueInputChanged(tab_id, s)))
        .on_submit(fog(FogDataMessage::ValueSubmitted(tab_id)))
        .size(12);

    // Steppers — clamped at both ends so they can never produce an invalid
    // value.
    let minus = button(text(icon_char(Icon::Minus)).font(LUCIDE_FONT).size(11))
        .on_press_maybe(
            (value > 0).then(|| fog(FogDataMessage::FactorCommitted(tab_id, pair, value - 1))),
        )
        .padding([2, 6])
        .style(style::chip);

    let plus = button(text(icon_char(Icon::Plus)).font(LUCIDE_FONT).size(11))
        .on_press_maybe(
            (value < max).then(|| fog(FogDataMessage::FactorCommitted(tab_id, pair, value + 1))),
        )
        .padding([2, 6])
        .style(style::chip);

    let error_line: Element<'_, Message> = match &editor.input_error {
        Some(msg) => text(format!("⚠ {msg}")).size(11).style(error_text).into(),
        None => Space::new().into(),
    };

    container(
        row![
            text(readout).size(11).style(style::subtle_text),
            Space::new().width(12),
            minus,
            value_field,
            plus,
            Space::new().width(8),
            error_line,
            Space::new().width(Fill),
            text("drag across the curve to paint · click to select a pair")
                .size(10)
                .style(style::shortcut_hint),
        ]
        .spacing(5)
        .padding([6, 12])
        .align_y(Alignment::Center),
    )
    .width(Fill)
    .into()
}

// ── Revert confirmation ───────────────────────────────────────────────────────

fn view_revert_confirm(tab_id: usize) -> Element<'static, Message> {
    let body = column![
        text("Discard changes?").size(14).style(style::primary_text),
        text("The fade table has unsaved edits. Reloading from disk will discard every change in this tab.")
            .size(12)
            .style(style::subtle_text),
        row![
            Space::new().width(Fill),
            button(text("Cancel").size(12))
                .on_press(fog(FogDataMessage::RevertCancelled(tab_id)))
                .padding([5, 12])
                .style(style::playback_button),
            button(text("Discard & Reload").size(12))
                .on_press(fog(FogDataMessage::RevertConfirmed(tab_id)))
                .padding([5, 12])
                .style(style::export_button),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(10)
    .padding(20)
    .width(Length::Fixed(360.0));

    container(body).style(style::export_dialog_container).into()
}
