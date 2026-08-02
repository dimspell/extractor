use std::path::PathBuf;
use std::sync::Arc;

use hexedit::{
    app_update, app_view, HexEditorApp, HexEditorConfig, HexEditorMessage, HexEditorState,
};
use iced::widget::{button, column, container, row, text};
use iced::{Element, Fill, Font, Task};

fn main() -> iced::Result {
    iced::application(HexApp::new, HexApp::update, HexApp::view)
        .title("HexEdit")
        .run()
}

struct HexApp {
    app: HexEditorApp,
    status: String,
}

/// Binary-local message enum — named `Msg` to avoid clashing with
/// `hexedit::AppMessage`.
#[derive(Debug, Clone)]
enum Msg {
    OpenFiles,
    FilesPicked(Option<Vec<PathBuf>>),
    Hex(hexedit::AppMessage),
}

impl HexApp {
    fn new() -> (Self, Task<Msg>) {
        (
            Self {
                app: HexEditorApp::new(),
                status: "Open a file to start editing".to_string(),
            },
            Task::none(),
        )
    }

    /// Build the editor config from the active document. Saving flows through
    /// `config.on_save` inside `hexedit::update`, which clears the dirty flag
    /// and sets `state.status_msg` once `SavedIntoRecording` comes back.
    fn editor_config(&self) -> HexEditorConfig {
        let Some(active) = self.app.active_tab else {
            return HexEditorConfig::default();
        };
        if active >= self.app.documents.len() {
            return HexEditorConfig::default();
        }
        HexEditorConfig {
            pane_gap: 4,
            can_save: true,
            save_label: "Save".into(),
            extra_entries: Vec::new(),
            custom_encodings: Vec::new(),
            on_write_mode_changed: None,
            on_save: Some(Arc::new(|state: &HexEditorState| {
                let path = state.path.clone();
                let bytes = state.provider.as_slice().to_vec();
                Task::perform(
                    async move {
                        tokio::fs::write(&path, bytes)
                            .await
                            .map_err(|e| e.to_string())
                            .map(|_| "Saved".to_string())
                    },
                    HexEditorMessage::SavedIntoRecording,
                )
            })),
            ..HexEditorConfig::default()
        }
    }

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::OpenFiles => {
                let future = rfd::AsyncFileDialog::new()
                    .set_title("Open files for hex editing")
                    .pick_files();
                Task::perform(future, |opt| {
                    Msg::FilesPicked(opt.map(|handles| {
                        handles
                            .into_iter()
                            .map(|h| h.path().to_path_buf())
                            .collect()
                    }))
                })
            }
            Msg::FilesPicked(Some(paths)) => {
                if !paths.is_empty() {
                    let n = paths.len();
                    let config = self.editor_config();
                    let task = app_update(
                        &mut self.app,
                        &config,
                        hexedit::AppMessage::OpenFiles(paths),
                    );
                    self.status = format!("Opened {n} file(s)");
                    task.map(Msg::Hex)
                } else {
                    Task::none()
                }
            }
            Msg::FilesPicked(None) => {
                // User cancelled the dialog — nothing to do.
                Task::none()
            }
            Msg::Hex(msg) => {
                let config = self.editor_config();
                app_update(&mut self.app, &config, msg).map(Msg::Hex)
            }
        }
    }

    fn view(&self) -> Element<'_, Msg> {
        let menu_bar = row![
            button(text("Open Files").size(11).font(Font::MONOSPACE))
                .padding([3, 10])
                .on_press(Msg::OpenFiles),
            if let Some(active) = self.app.active_tab {
                if let Some(doc) = self.app.documents.get(active) {
                    if doc.state.provider.dirty_count() > 0 {
                        Some(
                            button(text("Save").size(11).font(Font::MONOSPACE))
                                .padding([3, 10])
                                .on_press(Msg::Hex(hexedit::AppMessage::Document(
                                    active,
                                    HexEditorMessage::SaveIntoRecording,
                                ))),
                        )
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            },
        ]
        .spacing(8);

        let status = text(&self.status).size(11).font(Font::MONOSPACE);

        let content: Element<'_, Msg> = if self.app.documents.is_empty() {
            container(
                column![
                    text("HexEdit").size(24).font(Font::MONOSPACE),
                    text("A standalone hex editor")
                        .size(12)
                        .font(Font::MONOSPACE),
                    button(text("Open Files").size(14).font(Font::MONOSPACE))
                        .padding([8, 24])
                        .on_press(Msg::OpenFiles),
                ]
                .spacing(16)
                .align_x(iced::Alignment::Center),
            )
            .width(Fill)
            .height(Fill)
            .align_x(iced::Alignment::Center)
            .into()
        } else {
            app_view(&self.app, &self.editor_config()).map(Msg::Hex)
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
