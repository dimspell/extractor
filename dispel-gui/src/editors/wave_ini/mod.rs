mod component;

use crate::app::App;
use crate::components::standard::message::StandardEditorMessage;
use crate::components::standard::StandardEditor;
use crate::handle_spreadsheet_messages;
use crate::message::MessageExt;
use dispel_core::WaveIni;
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

fn into_std(msg: WaveIniEditorMessage) -> StandardEditorMessage<WaveIni> {
    match msg {
        WaveIniEditorMessage::LoadCatalog => StandardEditorMessage::LoadCatalog,
        WaveIniEditorMessage::CatalogLoaded(r) => StandardEditorMessage::CatalogLoaded(r),
        WaveIniEditorMessage::Select(i) => StandardEditorMessage::Select(i),
        WaveIniEditorMessage::Save => StandardEditorMessage::Save,
        WaveIniEditorMessage::Saved(r) => StandardEditorMessage::Saved(r),
        WaveIniEditorMessage::PaneResized(e) => StandardEditorMessage::PaneResized(e),
        WaveIniEditorMessage::PaneClicked(p) => StandardEditorMessage::PaneClicked(p),
        _ => unreachable!(),
    }
}

fn wrap_std(msg: StandardEditorMessage<WaveIni>) -> crate::message::Message {
    crate::message::Message::wave_ini(match msg {
        StandardEditorMessage::LoadCatalog => WaveIniEditorMessage::LoadCatalog,
        StandardEditorMessage::CatalogLoaded(r) => WaveIniEditorMessage::CatalogLoaded(r),
        StandardEditorMessage::Select(i) => WaveIniEditorMessage::Select(i),
        StandardEditorMessage::FieldChanged(i, f, v) => WaveIniEditorMessage::FieldChanged(i, f, v),
        StandardEditorMessage::Spreadsheet(s) => WaveIniEditorMessage::Spreadsheet(s),
        StandardEditorMessage::PaneResized(e) => WaveIniEditorMessage::PaneResized(e),
        StandardEditorMessage::PaneClicked(p) => WaveIniEditorMessage::PaneClicked(p),
        StandardEditorMessage::Save => WaveIniEditorMessage::Save,
        StandardEditorMessage::Saved(r) => WaveIniEditorMessage::Saved(r),
    })
}

pub fn handle(message: WaveIniEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
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
        WaveIniEditorMessage::FieldChanged(index, field, value) => {
            let (old_value, orig_idx_u32) = app
                .state
                .editors.wave_ini_editor
                .filtered
                .iter()
                .find(|(i, _)| *i == index)
                .map(|(i, r)| {
                    use crate::components::editable::EditableRecord;
                    (r.get_field(&field), *i as u32)
                })
                .unwrap_or_default();
            let new_value = value.clone();
            let task = crate::components::standard::update::handle(
                StandardEditorMessage::FieldChanged(index, field.clone(), value),
                &mut app.state.editors.wave_ini_editor,
                &app.state.shared_game_path.clone(),
                &app.state.lookups,
                "Wave.ini",
                wrap_std,
            );
            let observe = if old_value != new_value {
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
            };
            observe.chain(task)
        }
        WaveIniEditorMessage::ExportWav(index) => {
            if app.state.shared_game_path.is_empty() {
                app.state.editors.wave_ini_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            if let Some((_, wave)) = app.state.editors.wave_ini_editor.filtered.get(index) {
                let snf_filename = match &wave.snf_filename {
                    Some(f) => f.clone(),
                    None => {
                        app.state.editors.wave_ini_editor.status_msg =
                            "No SNF filename for this entry.".into();
                        return Task::none();
                    }
                };
                let stem = std::path::Path::new(&snf_filename)
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| format!("wave_{}", wave.id));
                let game_path = app.state.shared_game_path.clone();
                app.state.editors.wave_ini_editor.loading_state =
                    crate::components::loading_state::LoadingState::Loading;
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
                    move |result| {
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
            app.state.editors.wave_ini_editor.loading_state =
                crate::components::loading_state::LoadingState::Loaded(());
            match result {
                Ok(p) => app.state.editors.wave_ini_editor.status_msg = format!("Exported to {}", p),
                Err(e) => app.state.editors.wave_ini_editor.status_msg = format!("Export failed: {}", e),
            }
            Task::none()
        }
        msg => crate::components::standard::update::handle(
            into_std(msg),
            &mut app.state.editors.wave_ini_editor,
            &app.state.shared_game_path.clone(),
            &app.state.lookups,
            "Wave.ini",
            wrap_std,
        ),
    }
}

pub fn view(app: &App) -> iced::Element<'_, crate::message::Message> {
    use crate::message::MessageExt;
    crate::view::editor::view_spreadsheet(
        &app.state.editors.wave_ini_editor,
        &app.state.editors.wave_ini_editor.spreadsheet,
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
        None,
        None,
    )
}
