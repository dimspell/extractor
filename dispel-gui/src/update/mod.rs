// Main update router
use crate::app::App;
use iced::Task;

impl App {
    pub fn update(&mut self, message: crate::message::Message) -> Task<crate::message::Message> {
        let task = match message {
            crate::message::Message::Workspace(msg) => workspace::handle(msg, self),
            crate::message::Message::Editor(msg) => editor::handle(msg, self),
            crate::message::Message::FileTree(msg) => file_tree::handle(msg, self),
            crate::message::Message::System(msg) => system::handle(msg, self),
            crate::message::Message::StartPage(msg) => startpage::handle(msg, self),
        };
        crate::components::composite_item::refresh_composite_item_options(&mut self.state);
        task
    }
}

// Domain-specific handler modules
pub mod editor;
mod file_tree;
mod startpage;
mod system;
mod workspace;

#[cfg(test)]
mod tests {
    use crate::app::App;
    use crate::components::composite_item::ItemOption;
    use crate::components::composite_item::refresh_composite_item_options;
    use crate::message::{Message, SystemMessage};
    use crate::workspace::Workspace;

    /// The refresh hook must run after routing, so a handler that empties the
    /// item lookup also empties the shared combo box options.
    #[test]
    fn test_update_refreshes_options_after_workspace_clear() {
        let mut app = App::test_new(Workspace::new());
        app.state.lookups.insert(
            "items".to_string(),
            vec![
                ("1:5".to_string(), "Invented Sword".to_string()),
                ("2:7".to_string(), "Invented Herb".to_string()),
            ],
        );
        refresh_composite_item_options(&mut app.state);
        assert_eq!(
            app.state.composite_item_options.state.options().len(),
            3,
            "the item catalog and the no-item sentinel must be loaded"
        );

        let _ = app.update(Message::System(SystemMessage::ClearWorkspace));

        assert!(
            app.state.lookups.is_empty(),
            "ClearWorkspace must empty the lookups"
        );
        let options: &[ItemOption] = app.state.composite_item_options.state.options();
        assert!(
            options.is_empty(),
            "App::update must rebuild the options after the handler, got {options:?}"
        );
    }
}
