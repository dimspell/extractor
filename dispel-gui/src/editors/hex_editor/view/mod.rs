pub mod footer;
pub mod inspector;
pub mod inspector_modal;
pub mod matrix;

use iced::widget::{column, container, row, text};
use iced::{Element, Fill, Font};

use crate::app::App;
use crate::components::modal::modal;
use crate::editors::hex_editor::HexEditorMessage;
use crate::editors::hex_editor::HexProvider;
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
    .into();

    let body = row![
        container(matrix).width(Fill).height(Fill),
        inspector::view(editor),
    ]
    .spacing(0);

    let base: Element<'_, Message> = column![
        header,
        container(body).width(Fill).height(Fill),
        footer::view(editor)
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
