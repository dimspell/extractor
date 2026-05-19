use std::path::PathBuf;

use iced::{color, Element, Task, Theme};

use hexedit::{view, HexEditorConfig, HexEditorMessage, HexEditorState};

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .theme(|_: &App| {
            Theme::custom(
                "HexEdit",
                iced::theme::Palette {
                    background: color!(0x2a2a2a),
                    text: color!(0xeae0c8),
                    primary: color!(0x8b5a2b),
                    success: color!(0x2d5a27),
                    danger: color!(0x800000),
                    warning: color!(0x8b8b00),
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

        let editor = match path {
            Some(p) => HexEditorState::load_from_path(&p),
            None => HexEditorState::load_from_path(
                &std::env::current_dir()
                    .unwrap_or_default()
                    .join("scratch.bin"),
            ),
        };

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
        self.config = HexEditorConfig {
            extra_entries: self.editor.lua_engine.entries(),
            can_save: has_dirty,
            save_label: "Save".to_string(),
            save_hint: if has_dirty {
                String::new()
            } else {
                "  ·  no edits".to_string()
            },
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
