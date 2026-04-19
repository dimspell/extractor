use std::path::{Path, PathBuf};

use crate::app::App;
use crate::message::editor::spritebrowser::{ExportFormat, SpriteViewerMessage};
use crate::state::sprite_viewer::{ExportDialogState, ExportStatus};
use iced::Task;

/// Real-time milliseconds per animation tick (~60 fps clock).
const TICK_MS: f32 = 16.0;

pub fn handle(message: SpriteViewerMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = app
        .state
        .workspace
        .active()
        .map(|t| t.id)
        .unwrap_or(usize::MAX);

    let Some(viewer) = app.state.sprite_viewers.get_mut(&tab_id) else {
        return Task::none();
    };

    match message {
        // ── Navigation ───────────────────────────────────────────────────────
        SpriteViewerMessage::SelectSequence(seq_idx) => {
            viewer.select_sequence(seq_idx);
        }
        SpriteViewerMessage::SelectFrame(frame_idx) => {
            viewer.select_frame(frame_idx);
        }
        SpriteViewerMessage::ScrubTo(frame_idx) => {
            viewer.is_playing = false;
            viewer.select_frame(frame_idx);
        }

        // ── Playback ─────────────────────────────────────────────────────────
        SpriteViewerMessage::Play => {
            viewer.is_playing = true;
        }
        SpriteViewerMessage::Pause => {
            viewer.is_playing = false;
        }
        SpriteViewerMessage::StepBack => {
            viewer.is_playing = false;
            let prev = viewer.selected_frame.saturating_sub(1);
            viewer.select_frame(prev);
        }
        SpriteViewerMessage::StepForward => {
            viewer.is_playing = false;
            let next = (viewer.selected_frame + 1).min(viewer.frames.len().saturating_sub(1));
            viewer.select_frame(next);
        }
        SpriteViewerMessage::ToggleLoop => {
            viewer.is_looping = !viewer.is_looping;
        }
        SpriteViewerMessage::SetSpeed(speed_100x) => {
            viewer.speed_100x = speed_100x;
        }
        SpriteViewerMessage::Tick => {
            viewer.tick(TICK_MS);
        }

        // ── Export dialog ────────────────────────────────────────────────────
        SpriteViewerMessage::ShowExportDialog => {
            viewer.export_dialog = Some(ExportDialogState::default());
        }
        SpriteViewerMessage::CloseExportDialog => {
            viewer.export_dialog = None;
        }
        SpriteViewerMessage::SetExportFormat(format) => {
            if let Some(ref mut dlg) = viewer.export_dialog {
                dlg.format = format;
                dlg.status = ExportStatus::Idle;
            }
        }
        SpriteViewerMessage::ChooseExportDir => {
            return Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .pick_folder()
                        .await
                        .map(|h| h.path().to_path_buf())
                },
                |path| {
                    crate::message::Message::Editor(crate::message::EditorMessage::SpriteViewer(
                        SpriteViewerMessage::ExportDirChosen(path),
                    ))
                },
            );
        }
        SpriteViewerMessage::ExportDirChosen(path) => {
            if let Some(ref mut dlg) = viewer.export_dialog {
                dlg.export_dir = path;
                dlg.status = ExportStatus::Idle;
            }
        }
        SpriteViewerMessage::ExportConfirm => {
            let Some(ref dlg) = viewer.export_dialog else {
                return Task::none();
            };
            let Some(ref export_dir) = dlg.export_dir else {
                return Task::none();
            };

            let format = dlg.format.clone();
            let sprite_name = viewer.name.clone();
            let export_dir = export_dir.clone();
            let frames: Vec<(usize, Vec<u8>)> = viewer
                .frames
                .iter()
                .map(|f| (f.frame_idx, f.png_bytes.clone()))
                .collect();

            if let Some(ref mut dlg) = viewer.export_dialog {
                dlg.status = ExportStatus::Exporting;
            }

            return Task::perform(
                perform_export(format, frames, sprite_name, export_dir),
                |result| {
                    crate::message::Message::Editor(crate::message::EditorMessage::SpriteViewer(
                        SpriteViewerMessage::ExportDone(result),
                    ))
                },
            );
        }
        SpriteViewerMessage::ExportDone(result) => {
            if let Some(ref mut dlg) = viewer.export_dialog {
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
    format: ExportFormat,
    frames: Vec<(usize, Vec<u8>)>,
    sprite_name: String,
    export_dir: PathBuf,
) -> Result<String, String> {
    match format {
        ExportFormat::PngFrames => export_png_frames(&frames, &sprite_name, &export_dir),
        ExportFormat::SpriteSheet => export_sprite_sheet(&frames, &sprite_name, &export_dir),
    }
}

fn export_png_frames(
    frames: &[(usize, Vec<u8>)],
    sprite_name: &str,
    export_dir: &Path,
) -> Result<String, String> {
    let dir = export_dir.join(sprite_name);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    for (idx, bytes) in frames {
        let path = dir.join(format!("frame_{:03}.png", idx));
        std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    }
    Ok(format!("Saved {} frames → {}", frames.len(), dir.display()))
}

fn export_sprite_sheet(
    frames: &[(usize, Vec<u8>)],
    sprite_name: &str,
    export_dir: &Path,
) -> Result<String, String> {
    if frames.is_empty() {
        return Err("No frames to export".to_string());
    }

    use image::DynamicImage;

    let decoded: Vec<DynamicImage> = frames
        .iter()
        .map(|(_, bytes)| image::load_from_memory(bytes).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    let total_width: u32 = decoded.iter().map(|img| img.width()).sum();
    let max_height: u32 = decoded.iter().map(|img| img.height()).max().unwrap_or(0);

    let mut sheet = image::RgbaImage::new(total_width, max_height);
    let mut x_offset = 0u32;
    for img in &decoded {
        let rgba = img.to_rgba8();
        for (x, y, px) in rgba.enumerate_pixels() {
            if x + x_offset < total_width && y < max_height {
                sheet.put_pixel(x + x_offset, y, *px);
            }
        }
        x_offset += img.width();
    }

    let path = export_dir.join(format!("{}_sheet.png", sprite_name));
    sheet.save(&path).map_err(|e| e.to_string())?;
    Ok(format!("Saved sprite sheet → {}", path.display()))
}
