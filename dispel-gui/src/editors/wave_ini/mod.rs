mod component;

use crate::app::App;
use crate::components::loading_state::LoadingState;
use crate::components::standard::StandardEditor;
use crate::handle_spreadsheet_messages;
use crate::message::MessageExt;
use dispel_core::{Extractor, WaveIni};
use iced::Task;

pub type WaveIniEditorState = StandardEditor<WaveIni>;

#[derive(Debug, Clone)]
pub enum WaveIniEditorMessage {
    LoadCatalog,
    CatalogLoaded(Result<Vec<WaveIni>, String>),
    Select(usize),
    FieldChanged(usize, String, String),
    Spreadsheet(crate::view::editor::SpreadsheetMessage),
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    PaneClicked(iced::widget::pane_grid::Pane),
    Save,
    Saved(Result<(), String>),
    ExportWav(usize),
    ExportedWav(Result<String, String>),
}

pub fn handle(message: WaveIniEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        WaveIniEditorMessage::LoadCatalog => {
            if app.state.shared_game_path.is_empty() {
                app.state.wave_ini_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.wave_ini_editor.loading_state = LoadingState::Loading;
            app.state.wave_ini_editor.spreadsheet.is_loading = true;
            let path = std::path::PathBuf::from(&app.state.shared_game_path).join("Wave.ini");
            Task::perform(
                async move { WaveIni::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                move |result| {
                    crate::message::Message::wave_ini(WaveIniEditorMessage::CatalogLoaded(result))
                },
            )
        }
        WaveIniEditorMessage::CatalogLoaded(result) => {
            app.state.wave_ini_editor.loading_state = LoadingState::Loaded(());
            match result {
                Ok(catalog) => {
                    app.state.wave_ini_editor.catalog = Some(catalog.clone());
                    app.state.wave_ini_editor.status_msg =
                        format!("Wave catalog loaded: {} entries", catalog.len());
                    app.state.wave_ini_editor.refresh();
                    app.state.wave_ini_editor.init_pane_state();
                    app.state.wave_ini_editor.spreadsheet.apply_filter(&catalog);
                    app.state
                        .wave_ini_editor
                        .spreadsheet
                        .compute_all_caches(&catalog);
                    app.state.wave_ini_editor.spreadsheet.is_loading = false;
                }
                Err(e) => {
                    app.state.wave_ini_editor.status_msg =
                        format!("Error loading wave catalog: {}", e);
                    app.state.wave_ini_editor.spreadsheet.is_loading = false;
                }
            }
            Task::none()
        }
        WaveIniEditorMessage::Select(index) => {
            app.state.wave_ini_editor.select(index);
            Task::none()
        }
        WaveIniEditorMessage::FieldChanged(index, field, value) => {
            let (old_value, orig_idx_u32) = app
                .state
                .wave_ini_editor
                .filtered
                .iter()
                .find(|(i, _)| *i == index)
                .map(|(i, r)| {
                    use crate::components::editable::EditableRecord;
                    (r.get_field(&field), *i as u32)
                })
                .unwrap_or_default();
            let new_value = value.clone();
            app.state.wave_ini_editor.update_field(index, &field, value);
            if old_value != new_value {
                crate::editors::mod_packager::recording::observe_field_change(
                    app,
                    "Wave.ini",
                    orig_idx_u32,
                    &field,
                    old_value,
                    new_value,
                )
            } else {
                Task::none()
            }
        }
        WaveIniEditorMessage::Save => {
            if app.state.shared_game_path.is_empty() {
                app.state.wave_ini_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.wave_ini_editor.loading_state = LoadingState::Loading;
            let result = app
                .state
                .wave_ini_editor
                .save(&app.state.shared_game_path, "Wave.ini");
            Task::perform(async { result }, |result| {
                crate::message::Message::wave_ini(WaveIniEditorMessage::Saved(result))
            })
        }
        WaveIniEditorMessage::Saved(result) => {
            app.state.wave_ini_editor.loading_state = LoadingState::Loaded(());
            match result {
                Ok(_) => {
                    app.state.wave_ini_editor.status_msg = "Wave ini saved successfully.".into()
                }
                Err(e) => {
                    app.state.wave_ini_editor.status_msg = format!("Error saving wave ini: {}", e)
                }
            }
            Task::none()
        }
        WaveIniEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages!(
                app,
                wave_ini_editor,
                |index, field, value| {
                    crate::message::Message::wave_ini(WaveIniEditorMessage::FieldChanged(
                        index, field, value,
                    ))
                },
                msg
            );
            Task::none()
        }
        WaveIniEditorMessage::ExportWav(index) => {
            if app.state.shared_game_path.is_empty() {
                app.state.wave_ini_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            if let Some((_, wave)) = app.state.wave_ini_editor.filtered.get(index) {
                let snf_filename = match &wave.snf_filename {
                    Some(f) => f.clone(),
                    None => {
                        app.state.wave_ini_editor.status_msg =
                            "No SNF filename for this entry.".into();
                        return Task::none();
                    }
                };
                let stem = std::path::Path::new(&snf_filename)
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| format!("wave_{}", wave.id));
                let game_path = app.state.shared_game_path.clone();
                app.state.wave_ini_editor.loading_state = LoadingState::Loading;
                return Task::perform(
                    async move {
                        let handle = rfd::AsyncFileDialog::new()
                            .set_file_name(format!("{}.wav", stem))
                            .add_filter("WAV Audio", &["wav"])
                            .save_file()
                            .await;
                        match handle {
                            Some(h) => {
                                let output_path = h.path().to_path_buf();
                                if let Some(parent) = output_path.parent() {
                                    let _ = std::fs::create_dir_all(parent);
                                }
                                let snf_path = App::find_snf_file(&game_path, &snf_filename);
                                dispel_core::snf::extract(&snf_path, &output_path)
                                    .map(|_| output_path.to_string_lossy().to_string())
                                    .map_err(|e| e.to_string())
                            }
                            None => Err("Export cancelled".into()),
                        }
                    },
                    move |result: Result<String, String>| {
                        crate::message::Message::Editor(
                            crate::message::editor::EditorMessage::WaveIni(
                                WaveIniEditorMessage::ExportedWav(result),
                            ),
                        )
                    },
                );
            }
            Task::none()
        }
        WaveIniEditorMessage::ExportedWav(result) => {
            app.state.wave_ini_editor.loading_state = LoadingState::Loaded(());
            match result {
                Ok(p) => app.state.wave_ini_editor.status_msg = format!("Exported to {}", p),
                Err(e) => app.state.wave_ini_editor.status_msg = format!("Export failed: {}", e),
            }
            Task::none()
        }
        WaveIniEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = app.state.wave_ini_editor.pane_state {
                ps.resize(event.split, event.ratio);
            }
            if let Some(ref mut ps) = app.state.wave_ini_editor.spreadsheet.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }
        WaveIniEditorMessage::PaneClicked(pane) => {
            app.state.wave_ini_editor.pane_focus = Some(pane);
            Task::none()
        }
    }
}

pub fn view(app: &App) -> iced::Element<'_, crate::message::Message> {
    use crate::message::MessageExt;
    crate::view::editor::view_spreadsheet(
        &app.state.wave_ini_editor,
        &app.state.wave_ini_editor.spreadsheet,
        crate::message::Message::wave_ini(WaveIniEditorMessage::LoadCatalog),
        crate::message::Message::wave_ini(WaveIniEditorMessage::Save),
        |idx| crate::message::Message::wave_ini(WaveIniEditorMessage::Select(idx)),
        |idx, field, value| {
            crate::message::Message::wave_ini(WaveIniEditorMessage::FieldChanged(idx, field, value))
        },
        |msg| crate::message::Message::wave_ini(WaveIniEditorMessage::Spreadsheet(msg)),
        &app.state.lookups,
        |event| crate::message::Message::wave_ini(WaveIniEditorMessage::PaneResized(event)),
        |pane| crate::message::Message::wave_ini(WaveIniEditorMessage::PaneClicked(pane)),
    )
}
