pub mod footer;
pub mod inspector;
pub mod matrix;

use iced::widget::{column, container, row, text};
use iced::{Element, Fill, Font};

use crate::app::App;
use crate::editors::hex_editor::HexEditorMessage;
use crate::editors::hex_editor::HexProvider;
use crate::message::{Message, MessageExt};
use crate::view::editor::ParagraphCache;

use self::matrix::HexMatrix;

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
    let matrix: Element<'_, Message> = HexMatrix::new(
        editor.provider.as_slice(),
        editor.bytes_per_row,
        editor.selection,
        cache,
    )
    .on_select_at(|addr| Message::hex_editor(HexEditorMessage::SelectAt(addr)))
    .on_extend_to(|addr| Message::hex_editor(HexEditorMessage::ExtendTo(addr)))
    .on_nav(|dir, extend| Message::hex_editor(HexEditorMessage::Nav { dir, extend }))
    .into();

    let body = row![
        container(matrix).width(Fill).height(Fill),
        inspector::view(editor),
    ]
    .spacing(0);

    column![
        header,
        container(body).width(Fill).height(Fill),
        footer::view(editor)
    ]
    .spacing(0)
    .width(Fill)
    .height(Fill)
    .into()
}
