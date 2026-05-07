pub mod matrix;

use iced::widget::{column, container, text};
use iced::{Element, Fill, Font};

use crate::app::App;
use crate::editors::hex_editor::HexProvider;
use crate::message::Message;
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

    // The widget needs a long-lived ParagraphCache; store one per app and
    // hand a cheap Arc-clone to the matrix each frame. For commit 1 we
    // materialise a fresh cache per view call — paragraphs are reused
    // within a frame even with a fresh cache (every glyph is shaped once
    // per draw call), and the next commit will lift the cache onto state
    // alongside selection / inspector data.
    let cache = ParagraphCache::default();
    let matrix: Element<'_, Message> =
        HexMatrix::new(editor.provider.as_slice(), editor.bytes_per_row, cache).into();

    column![header, container(matrix).width(Fill).height(Fill)]
        .spacing(0)
        .width(Fill)
        .height(Fill)
        .into()
}
