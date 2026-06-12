pub mod export_modal;
pub mod footer;
pub mod goto_modal;
pub mod inspector;
pub mod inspector_modal;
pub mod matrix;
pub mod patterns;
pub mod repeat_modal;
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
        &state.row_annotations,
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

    // Use cursor (not context_menu_addr) so the check is accurate even on
    // the first frame: left-click moves cursor synchronously before view runs,
    // and right-click also publishes SelectAt to move cursor.
    let have_pattern_at_addr = state.pattern_id_at(state.selection.cursor).is_some();
    let pattern_at_cursor = have_pattern_at_addr
        .then(|| state.pattern_id_at(state.selection.cursor))
        .flatten()
        .and_then(|pid| state.pattern_by_id(pid));
    let pattern_group_at_cursor = pattern_at_cursor
        .and_then(|p| p.group_id)
        .and_then(|gid| state.groups.iter().find(|g| g.id == gid));
    let group_id_at_cursor = pattern_group_at_cursor.map(|g| g.id);
    let pattern_menu_entries = build_pattern_menu_entries(
        !state.selection.is_single(),
        !state.patterns.is_empty(),
        have_pattern_at_addr,
        group_id_at_cursor,
    );

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

    let base = if let Some(ref g) = state.goto {
        modal(
            base,
            goto_modal::view(g),
            || HexEditorMessage::CloseGotoDialog,
            0.3,
        )
    } else {
        base
    };

    let base = if let Some(ref rp) = state.repeat_pattern {
        modal(
            base,
            repeat_modal::view(rp),
            || HexEditorMessage::CloseRepeatedPattern,
            0.3,
        )
    } else {
        base
    };

    if state.export_config.is_some() {
        modal(
            base,
            export_modal::view(state),
            || HexEditorMessage::CloseExportConfig,
            0.35,
        )
    } else {
        base
    }
}

/// Build the pattern menu entries for the context menu.
///
/// Extracted as a pure function so the enabled/disabled logic can be
/// unit-tested without simulating UI interactions.
///
/// - `has_selection_range`: `true` if there is a multi-byte selection (enables
///   "Create Pattern" and "Add Repeated Pattern")
/// - `has_patterns`: `true` if any patterns exist
/// - `have_pattern_at_addr`: `true` if the right-click address falls within a
///   pattern (enables "Remove Pattern")
pub(crate) fn build_pattern_menu_entries(
    has_selection_range: bool,
    has_patterns: bool,
    have_pattern_at_addr: bool,
    group_id_at_cursor: Option<usize>,
) -> Vec<MenuEntry<HexEditorMessage>> {
    let mut entries = Vec::new();
    if has_selection_range {
        entries.push(MenuEntry::item(
            "Create Pattern",
            HexEditorMessage::CreatePattern,
        ));
        entries.push(MenuEntry::item(
            "Add Repeated Pattern",
            HexEditorMessage::BeginRepeatedPattern,
        ));
    } else {
        entries.push(MenuEntry::disabled("Create Pattern"));
        entries.push(MenuEntry::disabled("Add Repeated Pattern"));
    }
    if has_patterns {
        if have_pattern_at_addr {
            entries.push(MenuEntry::item(
                "Remove Pattern",
                HexEditorMessage::RemovePatternAtContextMenu,
            ));
        } else {
            entries.push(MenuEntry::disabled("Remove Pattern"));
        }
        if let Some(gid) = group_id_at_cursor {
            entries.push(MenuEntry::separator());
            entries.push(MenuEntry::item(
                "Remove Group",
                HexEditorMessage::RemovePatternGroup(gid),
            ));
        }
    }
    if has_patterns {
        entries.push(MenuEntry::item(
            "Clear All Patterns",
            HexEditorMessage::ClearAllPatterns,
        ));
    } else {
        entries.push(MenuEntry::disabled("Clear All Patterns"));
    }
    entries.push(MenuEntry::separator());
    if has_patterns {
        entries.push(MenuEntry::item(
            "Export Patterns…",
            HexEditorMessage::ExportPatterns,
        ));
    } else {
        entries.push(MenuEntry::disabled("Export Patterns…"));
    }
    entries.push(MenuEntry::item(
        "Import Patterns…",
        HexEditorMessage::ImportPatterns,
    ));
    entries
}

