// Main update router
use crate::app::App;
use iced::Task;

impl App {
    pub fn update(&mut self, message: crate::message::Message) -> Task<crate::message::Message> {
        match message {
            crate::message::Message::Workspace(msg) => workspace::handle(msg, self),
            crate::message::Message::Editor(msg) => editor::handle(msg, self),
            crate::message::Message::FileTree(msg) => file_tree::handle(msg, self),
            crate::message::Message::Viewer(msg) => {
                crate::editors::db_viewer::update::handle(msg, self)
            }
            crate::message::Message::System(msg) => system::handle(msg, self),
            crate::message::Message::StartPage(msg) => startpage::handle(msg, self),
            crate::message::Message::MapPreview(msg) => {
                // Route map preview messages to the active save file viewer
                let tab_id = match self.state.workspace.active() {
                    Some(t) => t.id,
                    None => return Task::none(),
                };
                let state = match self.state.editors.save_file_viewers.get_mut(&tab_id) {
                    Some(s) => s,
                    None => return Task::none(),
                };
                let Some(preview) = state.map_preview.as_mut() else {
                    return Task::none();
                };
                crate::editors::save_file_viewer::map_preview::handle(msg, preview)
            }
        }
    }
}

// Domain-specific handler modules
pub mod editor;
mod file_tree;
mod startpage;
mod system;
mod workspace;
