pub mod footer;
pub mod goto_modal;
pub mod inspector;
pub mod inspector_modal;
pub mod matrix;
pub mod patterns;
pub mod search_overlay;

use iced::widget::space::Space;
use iced::widget::{button, column, container, row, text};
use iced::{Element, Fill, Font};

use crate::app::App;
use crate::components::context_menu::{ContextMenu, Entry as MenuEntry};
use crate::components::modal::modal;
use crate::editors::hex_editor::{HexEditorMessage, HexEditorState, HexProvider};
use crate::message::{Message, MessageExt};
use crate::view::editor::ParagraphCache;

use self::matrix::{EditView, HexMatrix};

pub fn view(app: &App) -> Element<'_, Message> {
    let tab_id = app
        .state
        .workspace
        .active()
        .map(|t| t.id)
        .unwrap_or(usize::MAX);

    let Some(editor) = app.state.hex_editors.get(&tab_id) else {
        return container(text("Hex editor not loaded").size(14))
            .width(Fill)
            .height(Fill)
            .padding(16)
            .into();
    };

    if let Some(ref err) = editor.error {
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

    let total = editor.provider.len();
    let header = container(
        text(format!(
            "{}  ·  {} bytes  ·  {} bytes/row",
            editor.name, total, editor.bytes_per_row
        ))
        .size(11)
        .font(Font::MONOSPACE),
    )
    .padding([6, 12])
    .width(Fill);

    let toolbar = build_toolbar(app, editor);

    let cache = ParagraphCache::default();
    let edit = editor.edit_mode.as_ref().map(|e| EditView {
        addr: e.addr,
        draft: e.draft.as_str(),
    });
    let matrix: Element<'_, Message> = HexMatrix::new(
        editor.provider.as_slice(),
        editor.bytes_per_row,
        editor.selection,
        edit,
        editor.provider.dirty(),
        &editor.vanilla_diff,
        &editor.pattern_by_addr,
        &editor.search.match_set,
        editor.search.query_len,
        editor.search.current_addr(),
        &editor.search.results,
        cache,
    )
    .on_select_at(|addr| Message::hex_editor(HexEditorMessage::SelectAt(addr)))
    .on_extend_to(|addr| Message::hex_editor(HexEditorMessage::ExtendTo(addr)))
    .on_nav(|dir, extend| Message::hex_editor(HexEditorMessage::Nav { dir, extend }))
    .on_begin_edit(|addr| Message::hex_editor(HexEditorMessage::BeginEdit(addr)))
    .on_edit_type(|c| Message::hex_editor(HexEditorMessage::EditTypeChar(c)))
    .on_edit_backspace(|| Message::hex_editor(HexEditorMessage::EditBackspace))
    .on_edit_cancel(|| Message::hex_editor(HexEditorMessage::EditCancel))
    .on_edit_commit(|advance| Message::hex_editor(HexEditorMessage::EditCommit { advance }))
    .on_right_click(|addr| Message::hex_editor(HexEditorMessage::RightClickAt(addr)))
    .on_create_pattern(|| Message::hex_editor(HexEditorMessage::CreatePattern))
    .on_open_goto(|| Message::hex_editor(HexEditorMessage::OpenGotoDialog))
    .on_open_search(|| Message::hex_editor(HexEditorMessage::OpenSearch))
    .show_decimal(editor.show_decimal)
    .on_toggle_addr_format(|| Message::hex_editor(HexEditorMessage::ToggleAddrFormat))
    .into();

    let has_selection_range = !editor.selection.is_single();
    let clicked_on_pattern = editor
        .context_menu_addr
        .map(|addr| editor.pattern_id_at(addr).is_some())
        .unwrap_or(false);
    let has_patterns = !editor.patterns.is_empty();

    let mut pattern_menu_entries: Vec<MenuEntry<Message>> = Vec::new();
    if has_selection_range {
        pattern_menu_entries.push(MenuEntry::item(
            "Create Pattern",
            Message::hex_editor(HexEditorMessage::CreatePattern),
        ));
    } else {
        pattern_menu_entries.push(MenuEntry::disabled("Create Pattern"));
    }
    if clicked_on_pattern {
        if let Some(addr) = editor.context_menu_addr {
            pattern_menu_entries.push(MenuEntry::item(
                "Remove Pattern",
                Message::hex_editor(HexEditorMessage::RemovePatternAt(addr)),
            ));
        }
    }
    if has_patterns {
        pattern_menu_entries.push(MenuEntry::item(
            "Clear All Patterns",
            Message::hex_editor(HexEditorMessage::ClearAllPatterns),
        ));
    } else {
        pattern_menu_entries.push(MenuEntry::disabled("Clear All Patterns"));
    }

    let matrix = ContextMenu::new(matrix, pattern_menu_entries);

    let body = row![
        container(matrix).width(Fill).height(Fill),
        inspector::view(editor),
    ]
    .spacing(0);

    let pattern_section: Element<'_, Message> = if editor.show_pattern_list {
        patterns::view(editor)
    } else {
        Space::default().height(0).into()
    };

    let search_section: Element<'_, Message> = if editor.search.is_visible() {
        search_overlay::view(&editor.search)
    } else {
        Space::default().height(0).into()
    };

    let base: Element<'_, Message> = column![
        toolbar,
        search_section,
        header,
        pattern_section,
        container(body).width(Fill).height(Fill),
        footer::view(editor),
    ]
    .spacing(0)
    .width(Fill)
    .height(Fill)
    .into();

    let base = if let Some(ref ie) = editor.inspector_edit {
        modal(
            base,
            inspector_modal::view(ie),
            || Message::hex_editor(HexEditorMessage::CloseInspectorEdit),
            0.4,
        )
    } else {
        base
    };

    if let Some(ref g) = editor.goto {
        modal(
            base,
            goto_modal::view(g),
            || Message::hex_editor(HexEditorMessage::CloseGotoDialog),
            0.3,
        )
    } else {
        base
    }
}

fn build_toolbar<'a>(app: &'a App, editor: &'a HexEditorState) -> Element<'a, Message> {
    let recording = app.state.recording.as_ref();
    let has_dirty = editor.provider.dirty_count() > 0;
    let has_session = recording.is_some();
    let has_game = app.state.workspace.game_path.is_some();
    let in_game_dir = app
        .state
        .workspace
        .game_path
        .as_ref()
        .map(|gp| editor.path.starts_with(gp))
        .unwrap_or(false);
    let can_save = has_dirty && has_session && has_game && in_game_dir;

    let label = match recording {
        Some(s) => format!("Save into `{}`", s.mod_slug),
        None => "Save into recording".to_string(),
    };
    let mut save_btn = button(text(label).size(11).font(Font::MONOSPACE)).padding([3, 10]);
    if can_save {
        save_btn = save_btn.on_press(Message::hex_editor(HexEditorMessage::SaveIntoRecording));
    }

    let hint = if !has_session {
        "  ·  no recording active"
    } else if !has_game {
        "  ·  set a game directory"
    } else if !in_game_dir {
        "  ·  file is outside the game directory"
    } else if !has_dirty {
        "  ·  no edits to save"
    } else {
        ""
    };

    let patterns_label = if editor.show_pattern_list {
        "Hide Patterns"
    } else {
        "Patterns"
    };
    let patterns_btn = button(text(patterns_label).size(11).font(Font::MONOSPACE))
        .padding([3, 10])
        .on_press(Message::hex_editor(HexEditorMessage::TogglePatternList));

    // Bytes-per-row toggle group.
    let goto_btn = button(text("Go to...").size(11).font(Font::MONOSPACE))
        .padding([3, 10])
        .on_press(Message::hex_editor(HexEditorMessage::OpenGotoDialog));

    let bpr = editor.bytes_per_row;
    let bpr_btn = |n: u8| {
        let label = format!("{:02}", n);
        let active = bpr == n;
        let mut btn = button(text(label).size(11).font(Font::MONOSPACE)).padding([3, 6]);
        if !active {
            btn = btn.style(button::text);
        }
        btn.on_press(Message::hex_editor(HexEditorMessage::SetBytesPerRow(n)))
    };

    let status: Element<'a, Message> = if editor.status_msg.is_empty() {
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
