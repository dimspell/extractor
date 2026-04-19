use crate::app::App;
use crate::message::editor::tileset::{TileExportFormat, TilesetEditorMessage};
use crate::state::tileset_editor::{ExportDialogState, ExportStatus};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: TilesetEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = app
        .state
        .workspace
        .active()
        .map(|t| t.id)
        .unwrap_or(usize::MAX);

    let Some(editor) = app.state.tileset_editors.get_mut(&tab_id) else {
        return Task::none();
    };

    match message {
        TilesetEditorMessage::SetZoom(zoom) => {
            editor.zoom = zoom.clamp(0.5, 4.0);
        }

        // ── Export dialog ────────────────────────────────────────────────────
        TilesetEditorMessage::ShowExportDialog => {
            editor.export_dialog = Some(ExportDialogState::default());
        }
        TilesetEditorMessage::CloseExportDialog => {
            editor.export_dialog = None;
        }
        TilesetEditorMessage::SetExportFormat(format) => {
            if let Some(ref mut dlg) = editor.export_dialog {
                dlg.format = format;
                dlg.status = ExportStatus::Idle;
            }
        }
        TilesetEditorMessage::ChooseExportDir => {
            return Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .pick_folder()
                        .await
                        .map(|h| h.path().to_path_buf())
                },
                |path| {
                    crate::message::Message::Editor(crate::message::EditorMessage::Tileset(
                        TilesetEditorMessage::ExportDirChosen(path),
                    ))
                },
            );
        }
        TilesetEditorMessage::ExportDirChosen(path) => {
            if let Some(ref mut dlg) = editor.export_dialog {
                dlg.export_dir = path;
                dlg.status = ExportStatus::Idle;
            }
        }
        TilesetEditorMessage::ExportConfirm => {
            let Some(ref dlg) = editor.export_dialog else {
                return Task::none();
            };
            let Some(ref export_dir) = dlg.export_dir else {
                return Task::none();
            };

            let format = dlg.format.clone();
            let source_path = editor.path.clone();
            let tile_name = editor.name.clone();
            let export_dir = export_dir.clone();

            if let Some(ref mut dlg) = editor.export_dialog {
                dlg.status = ExportStatus::Exporting;
            }

            return Task::perform(
                perform_export(format, source_path, tile_name, export_dir),
                |result| {
                    crate::message::Message::Editor(crate::message::EditorMessage::Tileset(
                        TilesetEditorMessage::ExportDone(result),
                    ))
                },
            );
        }
        TilesetEditorMessage::ExportDone(result) => {
            if let Some(ref mut dlg) = editor.export_dialog {
                dlg.status = match result {
                    Ok(msg) => ExportStatus::Done(msg),
                    Err(e) => ExportStatus::Error(e),
                };
            }
        }
    }

    Task::none()
}

// ── Export logic ──────────────────────────────────────────────────────────────

async fn perform_export(
    format: TileExportFormat,
    source_path: PathBuf,
    tile_name: String,
    export_dir: PathBuf,
) -> Result<String, String> {
    let tiles = dispel_core::map::tileset::extract(&source_path).map_err(|e| e.to_string())?;

    match format {
        TileExportFormat::SeparateTiles => {
            let out_dir = export_dir.join(&tile_name);
            std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
            dispel_core::map::tileset::plot_all_tiles(&tiles, &out_dir.to_string_lossy());
            Ok(format!(
                "Saved {} tiles → {}",
                tiles.len(),
                out_dir.display()
            ))
        }
        TileExportFormat::Atlas => {
            let out_path = export_dir.join(format!("{}_atlas.png", tile_name));
            dispel_core::map::tileset::plot_tileset_map(&tiles, &out_path.to_string_lossy());
            Ok(format!("Saved atlas → {}", out_path.display()))
        }
    }
}
