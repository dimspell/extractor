pub mod footer;
pub mod inspector;
pub mod inspector_modal;
pub mod matrix;

use iced::widget::{button, column, container, row, text};
use iced::{Element, Fill, Font};

use crate::app::App;
use crate::components::context_menu::ContextMenu;
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
        &editor.patterns,
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
    .into();

    let has_selection_range = !editor.selection.is_single();
    let clicked_on_pattern = editor
        .context_menu_addr
        .map(|addr| editor.patterns.contains(&addr))
        .unwrap_or(false);
    let has_patterns = !editor.patterns.is_empty();

    let mut pattern_menu_entries: Vec<(&str, Message)> = Vec::new();
    if has_selection_range {
        pattern_menu_entries.push((
            "Create Pattern",
            Message::hex_editor(HexEditorMessage::CreatePattern),
        ));
    }
    if clicked_on_pattern {
        if let Some(addr) = editor.context_menu_addr {
            pattern_menu_entries.push((
                "Remove Pattern",
                Message::hex_editor(HexEditorMessage::RemovePatternAt(addr)),
            ));
        }
    }
    if has_patterns {
        pattern_menu_entries.push((
            "Clear All Patterns",
            Message::hex_editor(HexEditorMessage::ClearAllPatterns),
        ));
    }

    let matrix = if pattern_menu_entries.is_empty() {
        matrix
    } else {
        ContextMenu::new(matrix, pattern_menu_entries).into()
    };

    let body = row![
        container(matrix).width(Fill).height(Fill),
        inspector::view(editor),
    ]
    .spacing(0);

    let base: Element<'_, Message> = column![
        toolbar,
        header,
        container(body).width(Fill).height(Fill),
        footer::view(editor),
    ]
    .spacing(0)
    .width(Fill)
    .height(Fill)
    .into();

    if let Some(ref ie) = editor.inspector_edit {
        modal(
            base,
            inspector_modal::view(ie),
            || Message::hex_editor(HexEditorMessage::CloseInspectorEdit),
            0.4,
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
