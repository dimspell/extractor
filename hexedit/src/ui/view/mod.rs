pub mod footer;
pub mod goto_modal;
pub mod inspector;
pub mod inspector_modal;
pub mod matrix;
pub mod patterns;
pub mod search_overlay;
pub mod toolbar;

use gui_widgets::components::context_menu::{ContextMenu, Entry as MenuEntry};
use gui_widgets::components::modal::modal;
use iced::widget::space::Space;
use iced::widget::{column, container, row, text};
use iced::{Element, Fill, Font};

use crate::config::HexEditorConfig;
use crate::{HexEditorMessage, HexEditorState, HexProvider};

use self::matrix::{EditView, HexMatrix};
use self::toolbar::build_toolbar;

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

    let cache = state.cache.clone();
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
    .on_copy_selection(|| HexEditorMessage::CopySelection)
    .on_paste(|| HexEditorMessage::Paste)
    .show_decimal(state.show_decimal)
    .on_toggle_addr_format(|| HexEditorMessage::ToggleAddrFormat)
    .into();

    let has_selection_range = !state.selection.is_single();
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
    if has_patterns {
        // "Remove Pattern" always appears when patterns exist. The action
        // targets the right-click address (via context_menu_addr), which is
        // set synchronously during event processing before the native menu
        // fires. If the right-clicked byte is not in a pattern the action is
        // a harmless no-op.
        pattern_menu_entries.push(MenuEntry::item(
            "Remove Pattern",
            HexEditorMessage::RemovePatternAtContextMenu,
        ));
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

