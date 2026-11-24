use std::path::PathBuf;
use std::sync::Arc;

use iced::widget::{button, column, container, row, text};
use iced::{Element, Fill, Font, Task, Theme};

use hexedit::ui::theme::ThemeVariant;
use hexedit::{
    AppMessage, EncodingEntry, HexEditorApp, HexEditorConfig, HexEditorDocument, HexEditorMessage,
    HexEditorState, SaveOutcome, SaveRequest, WriteMode, app_update, app_view, standalone_config,
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
    if let Err(e) = std::fs::create_dir_all(&p) {
        eprintln!("[hexedit] failed to create settings directory: {e}");
    }
    p.push("settings.json");
    p
}

fn load_settings() -> PersistedSettings {
    let path = settings_path();
    match std::fs::read_to_string(&path) {
        Ok(json) => match serde_json::from_str(&json) {
            Ok(settings) => settings,
            Err(e) => {
                eprintln!("[hexedit] invalid settings file, using defaults: {e}");
                PersistedSettings::default()
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => PersistedSettings::default(),
        Err(e) => {
            eprintln!("[hexedit] failed to read settings, using defaults: {e}");
            PersistedSettings::default()
        }
    }
}

/// Iced palette seed for the given hex-editor theme variant.
fn seed_for_variant(variant: ThemeVariant) -> iced::theme::palette::Seed {
    let palette = variant.theme();
    iced::theme::palette::Seed {
        background: palette.iced_bg,
        text: palette.iced_text,
        primary: palette.iced_primary,
        success: palette.iced_success,
        danger: palette.iced_danger,
        warning: palette.iced_warning,
    }
}

/// Theme variant of the active document, or Dark when no file is open.
fn active_variant(app: &App) -> ThemeVariant {
    app.app
        .active_tab
        .and_then(|idx| app.app.documents.get(idx))
        .map(|doc| doc.state.theme_variant)
        .unwrap_or_default()
}

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("HexEdit")
        .theme(|app: &App| Theme::custom("HexEdit", seed_for_variant(active_variant(app))))
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
        let paths: Vec<PathBuf> = std::env::args().skip(1).map(PathBuf::from).collect();

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
    /// the `config.on_save` `SavePort` inside `hexedit::update`, which maps
    /// the returned `SaveOutcome` onto its internal `SavedIntoRecording`
    /// message, clears the dirty flag and sets `state.status_msg`.
    fn editor_config(&self) -> HexEditorConfig {
        let Some(active) = self.app.active_tab else {
            return HexEditorConfig::default();
        };
        let Some(doc) = self.app.documents.get(active) else {
            return HexEditorConfig::default();
        };

        let has_dirty = doc.state.provider.dirty_count() > 0;
        let settings_path = settings_path();

        let save_port: hexedit::SavePort = Arc::new(|request: SaveRequest| {
            let SaveRequest { path, bytes } = request;
            Task::perform(
                async move {
                    tokio::fs::write(&path, bytes)
                        .await
                        .map_err(|e| e.to_string())
                        .map(|_| "Saved".to_string())
                },
                SaveOutcome::from_result,
            )
        });

        let on_write_mode_changed: hexedit::WriteModeCallback = Arc::new(move |mode, encodings| {
            // Persist the write mode + the encodings current at callback time,
            // so custom-encoding changes are not lost on restart.
            let settings = PersistedSettings {
                write_mode: mode,
                custom_encodings: encodings.to_vec(),
            };
            match serde_json::to_string_pretty(&settings) {
                Ok(json) => {
                    if let Err(e) = std::fs::write(&settings_path, json) {
                        eprintln!("[hexedit] failed to persist settings: {e}");
                    }
                }
                Err(e) => eprintln!("[hexedit] failed to serialize settings: {e}"),
            }
        });

        standalone_config(
            has_dirty,
            doc.state.custom_encodings.clone(),
            Vec::new(),
            save_port,
            Some(on_write_mode_changed),
        )
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

#[cfg(test)]
mod tests {
    use super::*;
    use hexedit::ui::theme::{DARK_THEME, LIGHT_THEME};

    #[test]
    fn test_seed_for_variant_uses_that_palette() {
        let dark = seed_for_variant(ThemeVariant::Dark);
        let light = seed_for_variant(ThemeVariant::Light);
        assert_eq!(dark.background, DARK_THEME.iced_bg);
        assert_eq!(dark.text, DARK_THEME.iced_text);
        assert_eq!(light.background, LIGHT_THEME.iced_bg);
        assert_eq!(light.text, LIGHT_THEME.iced_text);
        assert_ne!(dark.background, light.background);
    }

    #[test]
    fn test_active_variant_defaults_to_dark_without_documents() {
        let mut app = App {
            app: HexEditorApp::new(),
            status: String::new(),
        };
        assert_eq!(active_variant(&app), ThemeVariant::Dark);
        app.app.documents.push(HexEditorDocument {
            state: HexEditorState::from_bytes("Invented.bin", vec![], None, None),
            pinned: false,
        });
        app.app.active_tab = Some(0);
        app.app.documents[0].state.theme_variant = ThemeVariant::Light;
        assert_eq!(active_variant(&app), ThemeVariant::Light);
    }
}
