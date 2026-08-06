use std::path::PathBuf;
use std::sync::Arc;

use iced::widget::{button, column, container, row, text};
use iced::{Element, Fill, Font, Task, Theme};

use hexedit::ui::theme::DARK_THEME;
use hexedit::{
    AppMessage, EncodingEntry, HexEditorApp, HexEditorConfig, HexEditorDocument, HexEditorMessage,
    HexEditorState, WriteMode, app_update, app_view,
};

/// Settings persisted to `~/.config/hexedit/settings.json`.
#[derive(serde::Serialize, serde::Deserialize)]
struct PersistedSettings {
    write_mode: WriteMode,
    custom_encodings: Vec<EncodingEntry>,
}

impl Default for PersistedSettings {
    fn default() -> Self {
        Self {
            write_mode: WriteMode::Hex,
            custom_encodings: Vec::new(),
        }
    }
}

fn settings_path() -> PathBuf {
    let mut p = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    p.push("hexedit");
    let _ = std::fs::create_dir_all(&p);
    p.push("settings.json");
    p
}

fn load_settings() -> PersistedSettings {
    let path = settings_path();
    match std::fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => PersistedSettings::default(),
    }
}

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("HexEdit")
        .theme(|_: &App| {
            Theme::custom(
                "HexEdit",
                iced::theme::palette::Seed {
                    background: DARK_THEME.iced_bg,
                    text: DARK_THEME.iced_text,
                    primary: DARK_THEME.iced_primary,
                    success: DARK_THEME.iced_success,
                    danger: DARK_THEME.iced_danger,
                    warning: DARK_THEME.iced_warning,
                },
            )
        })
        .window_size((1100.0, 800.0))
        .run()
}

struct App {
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

impl App {
    fn new() -> (Self, Task<Msg>) {
        let mut paths: Vec<PathBuf> = Vec::new();
        let mut script_dirs: Vec<PathBuf> = Vec::new();
        let args: Vec<String> = std::env::args().collect();
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--script-dir" | "-s" => {
                    i += 1;
                    if i < args.len() {
                        script_dirs.push(PathBuf::from(&args[i]));
                    }
                }
                _ => {
                    paths.push(PathBuf::from(&args[i]));
                }
            }
            i += 1;
        }

        let persisted = load_settings();

        let mut app = HexEditorApp::new();
        // Only build documents when positional paths were given. With no paths
        // the app stays empty and the welcome screen is shown.
        if !paths.is_empty() {
            for p in &paths {
                app.documents.push(HexEditorDocument {
                    state: HexEditorState::load_from_path(p),
                    pinned: false,
                });
            }
            app.active_tab = Some(0);

            // Apply persisted settings to every document.
            for doc in &mut app.documents {
                doc.state.write_mode = persisted.write_mode;
                doc.state.custom_encodings = persisted.custom_encodings.clone();
            }

            // Load Lua scripts into every document.
            for dir in &script_dirs {
                for doc in &mut app.documents {
                    let errors = doc.state.load_lua_scripts(dir);
                    for e in &errors {
                        eprintln!("[hexedit] script error: {e}");
                    }
                }
            }
        }

        (
            Self {
                app,
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
        let Some(doc) = self.app.documents.get(active) else {
            return HexEditorConfig::default();
        };

        let has_dirty = doc.state.provider.dirty_count() > 0;
        let settings_path = settings_path();
        // Clone the current custom encodings for the callback closure.
        let current_encodings = doc.state.custom_encodings.clone();
        HexEditorConfig {
            pane_gap: 4,
            can_save: true,
            save_label: "Save".to_string(),
            save_hint: if has_dirty {
                String::new()
            } else {
                "  ·  no edits".to_string()
            },
            extra_entries: doc.state.lua_engine.entries(),
            custom_encodings: current_encodings.clone(),
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
            on_write_mode_changed: Some(Arc::new(move |mode| {
                // Persist the write mode + current custom encodings.
                let settings = PersistedSettings {
                    write_mode: mode,
                    custom_encodings: current_encodings.clone(),
                };
                if let Ok(json) = serde_json::to_string_pretty(&settings) {
                    let _ = std::fs::write(&settings_path, json);
                }
                Task::none()
            })),
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
                    let task = app_update(&mut self.app, &config, AppMessage::OpenFiles(paths));
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
                                .on_press(Msg::Hex(AppMessage::Document(
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
