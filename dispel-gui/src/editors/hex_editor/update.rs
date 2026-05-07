use crate::app::App;
use crate::editors::hex_editor::selection::nav_target;
use crate::editors::hex_editor::HexEditorMessage;
use crate::editors::hex_editor::HexProvider;
use iced::Task;

/// Page nav heuristic — Commit 2 doesn't know the live viewport height, so we
/// default to a roughly screenful number of rows. The matrix widget will be
/// the source of truth in a later commit.
const PAGE_ROWS: u64 = 24;

pub fn handle(message: HexEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = app
        .state
        .workspace
        .active()
        .map(|t| t.id)
        .unwrap_or(usize::MAX);
    let Some(editor) = app.state.hex_editors.get_mut(&tab_id) else {
        return Task::none();
    };

    let max_addr = editor.max_addr();
    match message {
        HexEditorMessage::SetBytesPerRow(n) => {
            if matches!(n, 8 | 16 | 32) {
                editor.bytes_per_row = n;
            }
        }
        HexEditorMessage::SelectAt(addr) => {
            editor.selection.select(addr, max_addr);
        }
        HexEditorMessage::ExtendTo(addr) => {
            editor.selection.extend(addr, max_addr);
        }
        HexEditorMessage::Nav { dir, extend } => {
            if editor.provider.is_empty() {
                return Task::none();
            }
            let bpr = editor.bytes_per_row as u64;
            let target = nav_target(editor.selection.cursor, dir, bpr, PAGE_ROWS, max_addr);
            if extend {
                editor.selection.extend(target, max_addr);
            } else {
                editor.selection.select(target, max_addr);
            }
        }
    }
    Task::none()
}
