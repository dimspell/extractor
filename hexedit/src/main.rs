use std::path::PathBuf;
use std::sync::Arc;

use iced::widget::{container, text};
use iced::{Element, Fill, Font, Task, Theme};

use hexedit::ui::theme::DARK_THEME;
use hexedit::{
    app_update, app_view, AppMessage, EncodingEntry, HexEditorApp, HexEditorConfig,
    HexEditorDocument, HexEditorMessage, HexEditorState, WriteMode,
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
    config: HexEditorConfig,
}

impl App {
    fn new() -> (Self, Task<AppMessage>) {
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

        if paths.is_empty() {
            paths.push(
                std::env::current_dir()
                    .unwrap_or_default()
                    .join("scratch.bin"),
            );
        }

        let persisted = load_settings();

        let mut app = HexEditorApp::new();
        for p in &paths {
            app.documents.push(HexEditorDocument {
                state: HexEditorState::load_from_path(p),
                pinned: false,
            });
        }
        if !app.documents.is_empty() {
            app.active_tab = Some(0);
        }

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

        let mut this = Self {
            app,
            config: HexEditorConfig::default(),
        };
        this.refresh_config();

        (this, Task::none())
    }

    fn refresh_config(&mut self) {
        let Some(active) = self.app.active_tab else {
            self.config = HexEditorConfig::default();
            return;
        };
        let Some(doc) = self.app.documents.get(active) else {
            self.config = HexEditorConfig::default();
            return;
        };

        let has_dirty = doc.state.provider.dirty_count() > 0;
        let settings_path = settings_path();
        // Clone the current custom encodings for the callback closure.
        let current_encodings = doc.state.custom_encodings.clone();
        self.config = HexEditorConfig {
            extra_entries: doc.state.lua_engine.entries(),
            can_save: true,
            save_label: "Save".to_string(),
            save_hint: if has_dirty {
                String::new()
            } else {
                "  ·  no edits".to_string()
            },
            custom_encodings: current_encodings.clone(),
            on_save: Some(Arc::new(|state: &HexEditorState| {
                Task::done(
                    match std::fs::write(&state.path, state.provider.as_slice()) {
                        Ok(()) => HexEditorMessage::SavedIntoRecording(Ok("Saved".to_string())),
                        Err(e) => HexEditorMessage::SavedIntoRecording(Err(e.to_string())),
                    },
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
            ..HexEditorConfig::default()
        };
    }

    fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        let task = app_update(&mut self.app, &self.config, message);
        self.refresh_config();
        task
    }

    fn view(&self) -> Element<'_, AppMessage> {
        match self.app.active_tab {
            Some(active) if active < self.app.documents.len() => app_view(&self.app, &self.config),
            _ => container(
                text("No files open. Pass file paths as arguments to open them.")
                    .size(14)
                    .font(Font::MONOSPACE),
            )
            .width(Fill)
            .height(Fill)
            .align_x(iced::Alignment::Center)
            .align_y(iced::Alignment::Center)
            .into(),
        }
    }
}
