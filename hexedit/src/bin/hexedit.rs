use std::path::PathBuf;

use hexedit::{HexEditorConfig, HexEditorMessage, HexEditorState};
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Fill, Font, Task};

fn main() -> iced::Result {
    iced::application(HexApp::new, HexApp::update, HexApp::view)
        .title("HexEdit")
        .run()
}

struct HexApp {
    state: Option<HexEditorState>,
    path_input: String,
    status: String,
}

#[derive(Debug, Clone)]
enum AppMessage {
    Hex(HexEditorMessage),
    PathInput(String),
    OpenFile,
    Saved(Result<(), String>),
}

impl HexApp {
    fn new() -> (Self, Task<AppMessage>) {
        (
            Self {
                state: None,
                path_input: String::new(),
                status: "Enter a file path and press Open".to_string(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, msg: AppMessage) -> Task<AppMessage> {
        match msg {
            AppMessage::PathInput(s) => {
                self.path_input = s;
                Task::none()
            }
            AppMessage::OpenFile => {
                let path = PathBuf::from(&self.path_input);
                if !path.exists() {
                    self.status = format!("File not found: {}", path.display());
                    return Task::none();
                }
                self.state = Some(HexEditorState::load_from_path(&path));
                self.status = format!("Opened: {}", path.display());
                Task::none()
            }
            AppMessage::Saved(Ok(())) => {
                if let Some(ref mut s) = self.state {
                    s.provider.clear_dirty();
                }
                self.status = "Saved successfully".to_string();
                Task::none()
            }
            AppMessage::Saved(Err(e)) => {
                self.status = format!("Save failed: {e}");
                Task::none()
            }
            AppMessage::Hex(msg) => {
                let Some(state) = &mut self.state else {
                    return Task::none();
                };
                match msg {
                    HexEditorMessage::SaveIntoRecording => {
                        let path = state.path.clone();
                        let bytes = state.provider.as_slice().to_vec();
                        Task::perform(
                            async move {
                                tokio::fs::write(&path, bytes)
                                    .await
                                    .map_err(|e| e.to_string())
                            },
                            AppMessage::Saved,
                        )
                    }
                    HexEditorMessage::SavedIntoRecording(result) => {
                        match result {
                            Ok(m) => {
                                state.provider.clear_dirty();
                                state.status_msg = m;
                            }
                            Err(e) => {
                                state.status_msg = format!("Save failed: {e}");
                            }
                        }
                        Task::none()
                    }
                    _ => {
                        let can_save = state.provider.dirty_count() > 0;
                        let config = HexEditorConfig {
                            on_save: Some(std::sync::Arc::new(|_| {
                                Task::done(HexEditorMessage::SaveIntoRecording)
                            })),
                            save_label: "Save".to_string(),
                            can_save,
                            save_hint: String::new(),
                            extra_entries: Vec::new(),
                        };
                        hexedit::update(state, &config, msg).map(AppMessage::Hex)
                    }
                }
            }
        }
    }

    fn view(&self) -> Element<'_, AppMessage> {
        let menu_bar = row![
            text_input("Enter file path...", &self.path_input)
                .on_input(AppMessage::PathInput)
                .on_submit(AppMessage::OpenFile)
                .padding([3, 8])
                .size(11)
                .font(Font::MONOSPACE)
                .width(Fill),
            button(text("Open").size(11).font(Font::MONOSPACE))
                .padding([3, 10])
                .on_press(AppMessage::OpenFile),
            if let Some(ref s) = self.state {
                if s.provider.dirty_count() > 0 {
                    Some(
                        button(text("Save").size(11).font(Font::MONOSPACE))
                            .padding([3, 10])
                            .on_press(AppMessage::Hex(HexEditorMessage::SaveIntoRecording)),
                    )
                } else {
                    None
                }
            } else {
                None
            },
        ]
        .spacing(8);

        let status = text(&self.status)
            .size(11)
            .font(Font::MONOSPACE);

        let content: Element<'_, AppMessage> = match &self.state {
            Some(state) => {
                let can_save = state.provider.dirty_count() > 0;
                let config = HexEditorConfig {
                    on_save: Some(std::sync::Arc::new(|_| {
                        Task::done(HexEditorMessage::SaveIntoRecording)
                    })),
                    save_label: "Save".to_string(),
                    can_save,
                    save_hint: String::new(),
                    extra_entries: Vec::new(),
                };
                hexedit::view(state, &config)
                    .map(AppMessage::Hex)
            }
            None => container(
                container(
                    column![
                        text("HexEdit").size(24).font(Font::MONOSPACE),
                        text("A standalone hex editor")
                            .size(12)
                            .font(Font::MONOSPACE),
                    ]
                    .spacing(16),
                )
                .align_x(iced::Alignment::Center),
            )
            .width(Fill)
            .height(Fill)
            .into(),
        };

        column![
            container(menu_bar).padding([4, 12]).width(Fill),
            content,
            container(status).padding([4, 12]).width(Fill),
        ]
        .spacing(0)
        .width(Fill)
        .height(Fill)
        .into()
    }
}
