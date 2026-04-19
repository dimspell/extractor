// Main update router
use crate::app::App;
use iced::Task;

impl App {
    pub fn update(&mut self, message: crate::message::Message) -> Task<crate::message::Message> {
        match message {
            crate::message::Message::Workspace(msg) => workspace::handle(msg, self),
            crate::message::Message::Editor(msg) => editor::handle(msg, self),
            crate::message::Message::FileTree(msg) => file_tree::handle(msg, self),
            crate::message::Message::Viewer(msg) => viewer::handle(msg, self),
            crate::message::Message::System(msg) => system::handle(msg, self),
            crate::message::Message::StartPage(msg) => startpage::handle(msg, self),
        }
    }
}

// Domain-specific handler modules
mod editor;
mod file_tree;
mod startpage;
mod system;
mod viewer;
mod workspace;
