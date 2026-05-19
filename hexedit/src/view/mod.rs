pub mod footer;
pub mod goto_modal;
pub mod inspector;
pub mod inspector_modal;
pub mod matrix;
pub mod patterns;
pub mod search_overlay;

use gui_widgets::components::context_menu::{ContextMenu, Entry as MenuEntry};
use gui_widgets::components::modal::modal;
use gui_widgets::components::paragraph_cache::ParagraphCache;
use iced::widget::space::Space;
use iced::widget::{button, column, container, row, text};
use iced::{Element, Fill, Font};

use crate::config::HexEditorConfig;
use crate::{HexEditorMessage, HexEditorState, HexProvider};

use self::matrix::{EditView, HexMatrix};

pub fn view<'a>(
    state: &'a HexEditorState,
    config: &HexEditorConfig,
) -> Element<'a, HexEditorMessage> {
    if let Some(ref err) = state.error {
        return container(
            column![
                text("Failed to load file").size(14),
                text(err.as_str()).size(12).font(Font::MONOSPACE),
            ]
            .spacing(8),
        )
        .width(Fill)
        .height(Fill)
        .padding(16)
        .into();
    }

    let total = state.provider.len();
    let header = container(
        text(format!(
            "{}  ·  {} bytes  ·  {} bytes/row",
            state.name, total, state.bytes_per_row
        ))
        .size(11)
        .font(Font::MONOSPACE),
    )
    .padding([6, 12])
    .width(Fill);

    let toolbar = build_toolbar(state, config);

    let cache = ParagraphCache::default();
    let edit = state.edit_mode.as_ref().map(|e| EditView {
        addr: e.addr,
        draft: e.draft.as_str(),
    });
    let matrix: Element<'a, HexEditorMessage> = HexMatrix::new(
        state.provider.as_slice(),
        state.bytes_per_row,
        state.selection,
        edit,
        state.provider.dirty(),
        &state.vanilla_diff,
        &state.pattern_by_addr,
        &state.search.match_set,
        state.search.query_len,
        state.search.current_addr(),
        &state.search.results,
        cache,
    )
    .on_select_at(HexEditorMessage::SelectAt)
    .on_extend_to(HexEditorMessage::ExtendTo)
    .on_nav(|dir, extend| HexEditorMessage::Nav { dir, extend })
    .on_begin_edit(HexEditorMessage::BeginEdit)
    .on_edit_type(HexEditorMessage::EditTypeChar)
    .on_edit_backspace(|| HexEditorMessage::EditBackspace)
    .on_edit_cancel(|| HexEditorMessage::EditCancel)
    .on_edit_commit(|advance| HexEditorMessage::EditCommit { advance })
    .on_right_click(HexEditorMessage::RightClickAt)
    .on_create_pattern(|| HexEditorMessage::CreatePattern)
    .on_open_goto(|| HexEditorMessage::OpenGotoDialog)
    .on_open_search(|| HexEditorMessage::OpenSearch)
    .show_decimal(state.show_decimal)
    .on_toggle_addr_format(|| HexEditorMessage::ToggleAddrFormat)
    .into();

    let has_selection_range = !state.selection.is_single();
    let clicked_on_pattern = state
        .context_menu_addr
        .map(|addr| state.pattern_id_at(addr).is_some())
        .unwrap_or(false);
    let has_patterns = !state.patterns.is_empty();

    let mut pattern_menu_entries: Vec<MenuEntry<HexEditorMessage>> = Vec::new();
    if has_selection_range {
        pattern_menu_entries.push(MenuEntry::item(
            "Create Pattern",
            HexEditorMessage::CreatePattern,
        ));
    } else {
        pattern_menu_entries.push(MenuEntry::disabled("Create Pattern"));
    }
    if clicked_on_pattern {
        if let Some(addr) = state.context_menu_addr {
            pattern_menu_entries.push(MenuEntry::item(
                "Remove Pattern",
                HexEditorMessage::RemovePatternAt(addr),
            ));
        }
    }
    if has_patterns {
        pattern_menu_entries.push(MenuEntry::item(
            "Clear All Patterns",
            HexEditorMessage::ClearAllPatterns,
        ));
    } else {
        pattern_menu_entries.push(MenuEntry::disabled("Clear All Patterns"));
    }

    let matrix = ContextMenu::new(matrix, pattern_menu_entries);

    let body = row![
        container(matrix).width(Fill).height(Fill),
        inspector::view(state, config),
    ]
    .spacing(0);

    let pattern_section: Element<'a, HexEditorMessage> = if state.show_pattern_list {
        patterns::view(state)
    } else {
        Space::default().height(0).into()
    };

    let search_section: Element<'a, HexEditorMessage> = if state.search.is_visible() {
        search_overlay::view(&state.search)
    } else {
        Space::default().height(0).into()
    };

    let base: Element<'a, HexEditorMessage> = column![
        toolbar,
        search_section,
        header,
        pattern_section,
        container(body).width(Fill).height(Fill),
        footer::view(state),
    ]
    .spacing(0)
    .width(Fill)
    .height(Fill)
    .into();

    let base = if let Some(ref ie) = state.inspector_edit {
        modal(
            base,
            inspector_modal::view(ie),
            || HexEditorMessage::CloseInspectorEdit,
            0.4,
        )
    } else {
        base
    };

    if let Some(ref g) = state.goto {
        modal(
            base,
            goto_modal::view(g),
            || HexEditorMessage::CloseGotoDialog,
            0.3,
        )
    } else {
        base
    }
}

fn build_toolbar<'a>(
    editor: &'a HexEditorState,
    config: &HexEditorConfig,
) -> Element<'a, HexEditorMessage> {
    let can_save = config.can_save_now(editor);

    let save_label = config.save_label().to_string();
    let mut save_btn = button(text(save_label).size(11).font(Font::MONOSPACE)).padding([3, 10]);
    if can_save {
        save_btn = save_btn.on_press(HexEditorMessage::SaveIntoRecording);
    }

    let hint = config.save_hint.clone();

    let patterns_label = if editor.show_pattern_list {
        "Hide Patterns"
    } else {
        "Patterns"
    };
    let patterns_btn = button(text(patterns_label).size(11).font(Font::MONOSPACE))
        .padding([3, 10])
        .on_press(HexEditorMessage::TogglePatternList);

    // Bytes-per-row toggle group.
    let goto_btn = button(text("Go to...").size(11).font(Font::MONOSPACE))
        .padding([3, 10])
        .on_press(HexEditorMessage::OpenGotoDialog);

    let bpr = editor.bytes_per_row;
    let bpr_btn = |n: u8| {
        let label = format!("{:02}", n);
        let active = bpr == n;
        let mut btn = button(text(label).size(11).font(Font::MONOSPACE)).padding([3, 6]);
        if !active {
            btn = btn.style(button::text);
        }
        btn.on_press(HexEditorMessage::SetBytesPerRow(n))
    };

    let status: Element<'a, HexEditorMessage> = if editor.status_msg.is_empty() {
        text("").size(11).into()
    } else {
        text(editor.status_msg.clone())
            .size(11)
            .font(Font::MONOSPACE)
            .into()
    };

    container(
        row![
            save_btn,
            goto_btn,
            patterns_btn,
            row![
                text("BPR").size(10).font(Font::MONOSPACE),
                bpr_btn(8),
                bpr_btn(16),
                bpr_btn(32),
            ]
            .spacing(2)
            .align_y(iced::Alignment::Center),
            text(hint).size(11).font(Font::MONOSPACE),
            container(status).width(Fill),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    )
    .padding([4, 12])
    .width(Fill)
    .into()
}
