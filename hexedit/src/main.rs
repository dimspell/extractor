use std::path::PathBuf;
use std::sync::Arc;

use iced::{Element, Task, Theme};

use hexedit::{view, EncodingEntry, HexEditorConfig, HexEditorMessage, HexEditorState, WriteMode};
use hexedit::ui::theme::DARK_THEME;

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
                iced::theme::Palette {
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
    editor: HexEditorState,
    config: HexEditorConfig,
}

impl App {
    fn new() -> (Self, Task<HexEditorMessage>) {
        let mut path: Option<PathBuf> = None;
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
                    path = Some(PathBuf::from(&args[i]));
                }
            }
            i += 1;
        }

        let persisted = load_settings();

        let mut editor = match path {
            Some(p) => HexEditorState::load_from_path(&p),
            None => HexEditorState::load_from_path(
                &std::env::current_dir()
                    .unwrap_or_default()
                    .join("scratch.bin"),
            ),
        };

        // Apply persisted settings.
        editor.write_mode = persisted.write_mode;
        editor.custom_encodings = persisted.custom_encodings.clone();

        let mut app = Self {
            editor,
            config: HexEditorConfig::default(),
        };

        for dir in &script_dirs {
            let errors = app.editor.load_lua_scripts(dir);
            for e in &errors {
                eprintln!("[hexedit] script error: {e}");
            }
        }

        app.refresh_config();

        (app, Task::none())
    }

    fn refresh_config(&mut self) {
        let has_dirty = self.editor.provider.dirty_count() > 0;
        let settings_path = settings_path();
        // Clone the current custom encodings for the callback closure.
        let current_encodings = self.editor.custom_encodings.clone();
        self.config = HexEditorConfig {
            extra_entries: self.editor.lua_engine.entries(),
            can_save: has_dirty,
            save_label: "Save".to_string(),
            save_hint: if has_dirty {
                String::new()
            } else {
                "  ·  no edits".to_string()
            },
            custom_encodings: current_encodings.clone(),
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

    fn update(&mut self, message: HexEditorMessage) -> Task<HexEditorMessage> {
        let task = hexedit::update(&mut self.editor, &self.config, message);
        self.refresh_config();
        task
    }

    fn view(&self) -> Element<'_, HexEditorMessage> {
        view(&self.editor, &self.config)
    }
}
