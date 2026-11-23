use crate::app::App;
use crate::editors::tileset::{
    ExportDialogState, ExportStatus, TileExportFormat, TilesetEditorMessage,
};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: TilesetEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = app.state.workspace.active_tab_id().as_usize();

    let Some(editor) = app.state.editors.tileset_editors.get_mut(&tab_id) else {
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
            dispel_core::map::tileset::plot_all_tiles(&tiles, &out_dir.to_string_lossy())
                .map_err(|e| format!("Could not save tiles to {}: {e}", out_dir.display()))?;
            Ok(format!(
                "Saved {} tiles → {}",
                tiles.len(),
                out_dir.display()
            ))
        }
        TileExportFormat::Atlas => {
            let out_path = export_dir.join(format!("{}_atlas.png", tile_name));
            dispel_core::map::tileset::plot_tileset_map(&tiles, &out_path.to_string_lossy())
                .map_err(|e| format!("Could not save atlas to {}: {e}", out_path.display()))?;
            Ok(format!("Saved atlas → {}", out_path.display()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tileset_export_reports_unwritable_destination() {
        let base =
            std::env::temp_dir().join(format!("dispel_tileset_export_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();

        // A minimal, invented tileset: one 32x32 RGB565 tile.
        let source = base.join("invented_ground.gtl");
        std::fs::write(&source, vec![0u8; 32 * 32 * 2]).unwrap();

        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();

        // A directory where the PNG should go makes the save fail after the
        // destination directory itself was created successfully.
        let separate_dir = base.join("invented_separate");
        let tile_dir = separate_dir.join("invented_tiles");
        std::fs::create_dir_all(tile_dir.join("tile_0000.png")).unwrap();
        let result = runtime.block_on(perform_export(
            TileExportFormat::SeparateTiles,
            source.clone(),
            "invented_tiles".into(),
            separate_dir.clone(),
        ));
        let err = result.expect_err("an unwritable PNG destination must fail the export");
        assert!(
            err.contains(&tile_dir.display().to_string()),
            "the error must name the output path, got: {err}"
        );
        assert!(!err.starts_with("Saved"), "failure must not claim success");

        let atlas_dir = base.join("invented_atlas");
        std::fs::create_dir_all(atlas_dir.join("invented_tiles_atlas.png")).unwrap();
        let result = runtime.block_on(perform_export(
            TileExportFormat::Atlas,
            source.clone(),
            "invented_tiles".into(),
            atlas_dir.clone(),
        ));
        let err = result.expect_err("an unwritable atlas destination must fail the export");
        assert!(
            err.contains(&atlas_dir.display().to_string()),
            "the error must name the output path, got: {err}"
        );
        assert!(!err.starts_with("Saved"), "failure must not claim success");

        std::fs::remove_dir_all(&base).unwrap();
    }
}
