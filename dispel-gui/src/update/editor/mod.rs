// Editor message router — delegates to dispatch_table
use crate::app::App;
use crate::message::Message;
use crate::message::editor::EditorMessage;
use iced::Task;

pub fn handle(message: EditorMessage, app: &mut App) -> Task<Message> {
    // EventScr has uniform calling convention but is kept here
    // since the view path also has special wrapping; all others
    // go through the generated dispatch table.
    match message {
        EditorMessage::EventScr(msg) => crate::editors::event_scr::handle(msg, app),
        other => crate::dispatch_table::dispatch(other, app),
    }
}

// Common editor framework
mod common;
pub mod tab;
