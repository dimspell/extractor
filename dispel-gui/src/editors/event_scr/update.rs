use crate::app::App;
use crate::editors::event_scr::message::EventScrEditorMessage;
use crate::message::Message;
use dispel_core::references::event_scr::EventScript;
use iced::Task;

pub fn handle(message: EventScrEditorMessage, app: &mut App) -> Task<Message> {
    match message {
        EventScrEditorMessage::SectionChanged(section) => {
            app.state.event_scr_editor.set_current_section(section);
        }
        EventScrEditorMessage::VariableAdded(_index, _variable) => {
            if let Some(ref mut catalog) = app.state.event_scr_editor.catalog {
                if _index <= catalog.len() {
                    catalog.insert(_index, EventScript::default());
                }
            }
        }
        EventScrEditorMessage::VariableEdited(_index, _variable) => {
            // Edit variable logic
        }
        EventScrEditorMessage::VariableDeleted(_index) => {
            // Delete variable logic
        }
        EventScrEditorMessage::SpriteAdded(_index, _sprite) => {
            // Add sprite logic
        }
        EventScrEditorMessage::SpriteEdited(_index, _sprite) => {
            // Edit sprite logic
        }
        EventScrEditorMessage::SpriteDeleted(_index) => {
            // Delete sprite logic
        }
        EventScrEditorMessage::ActionAdded(_index, _action) => {
            // Add action logic
        }
        EventScrEditorMessage::ActionEdited(_index, _action) => {
            // Edit action logic
        }
        EventScrEditorMessage::ActionDeleted(_index) => {
            // Delete action logic
        }
        EventScrEditorMessage::Loaded(script) => {
            app.state.event_scr_editor.catalog = Some(vec![script]);
        },
        EventScrEditorMessage::LoadError(e) => {
            eprintln!("Failed to load EventScript: {}", e);
        },
        EventScrEditorMessage::Saved => {
            app.state.event_scr_editor.edit_history.clear();
        },
        EventScrEditorMessage::SaveError(e) => {
            eprintln!("Failed to save EventScript: {}", e);
        },
    }
    Task::none()
}

// Helper to load EventScript from path
pub fn load_from_path(path: std::path::PathBuf) -> Task<Message> {
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                <EventScript as dispel_core::Extractor>::read_file(&path)
                    .map(|mut scripts| {
                        if let Some(script) = scripts.pop() {
                            Ok(script)
                        } else {
                            Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "No EventScript found in file",
                            ))
                        }
                    })
                    .and_then(|res| res)
            })
            .await
            .unwrap_or_else(|e| Err(std::io::Error::new(std::io::ErrorKind::Other, e)))
        },
        |result| {
            Message::Editor(crate::message::editor::EditorMessage::EventScr(
                match result {
                    Ok(script) => EventScrEditorMessage::Loaded(script),
                    Err(e) => EventScrEditorMessage::LoadError(e.to_string()),
                },
            ))
        },
    )
}

// Helper to save EventScript to path
pub fn save_to_path(path: std::path::PathBuf, script: EventScript) -> Task<Message> {
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                let mut file = std::fs::File::create(&path)?;
                <EventScript as dispel_core::Extractor>::to_writer(&[script], &mut file)
            })
            .await
            .unwrap_or_else(|e| Err(std::io::Error::new(std::io::ErrorKind::Other, e)))
        },
        |result| {
            Message::Editor(crate::message::editor::EditorMessage::EventScr(
                match result {
                    Ok(()) => EventScrEditorMessage::Saved,
                    Err(e) => EventScrEditorMessage::SaveError(e.to_string()),
                },
            ))
        },
    )
}
