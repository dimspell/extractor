use std::path::{Path, PathBuf};

use crate::app::App;
use crate::editors::sprite_editor::{
    ExportDialogState, ExportFormat, ExportStatus, SpriteViewerMessage,
};
use dispel_core::sprite::{SpriteFrameData, write_sprite_to_path};
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

    let Some(viewer) = app.state.editors.sprite_viewers.get_mut(&tab_id) else {
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
            let max_frames = viewer
                .frame_counts
                .get(viewer.selected_sequence)
                .copied()
                .unwrap_or(0);
            let next = (viewer.selected_frame + 1).min(max_frames.saturating_sub(1));
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

        // ── Save ─────────────────────────────────────────────────────────────
        SpriteViewerMessage::Save => {
            let path = viewer.save_path.clone();
            let sf = viewer.sprite_file.clone();
            let recording = app.state.recording.as_ref().map(|session| {
                let game_path = std::path::PathBuf::from(&app.state.shared_game_path);
                let rel =
                    crate::editors::sprite_editor::message::RecordingParams::relative_path_for(
                        &path, &game_path,
                    )
                    .unwrap_or_default();
                crate::editors::sprite_editor::message::RecordingParams {
                    workspace_root: session.workspace_root.clone(),
                    game_path,
                    mod_slug: session.mod_slug.clone(),
                    relative_path: rel,
                }
            });
            return Task::perform(
                async move { save_sprite_file(path, sf, recording).await },
                |result| {
                    crate::message::Message::Editor(crate::message::EditorMessage::SpriteViewer(
                        SpriteViewerMessage::SaveComplete(result),
                    ))
                },
            );
        }
        SpriteViewerMessage::SaveComplete(result) => {
            match result {
                Ok(_msg) => {
                    viewer.mark_clean();
                    if let Some(tab) = app.state.workspace.tabs.iter_mut().find(|t| t.id == tab_id)
                    {
                        tab.modified = false;
                    }
                    // Keep path fresh
                    viewer.path = viewer.save_path.clone();
                }
                Err(e) => {
                    viewer.error = Some(e);
                }
            }
        }

        // ── Zoom ─────────────────────────────────────────────────────────────
        SpriteViewerMessage::ZoomIn => {
            viewer.zoom = (viewer.zoom * 1.25).min(10.0);
        }
        SpriteViewerMessage::ZoomOut => {
            viewer.zoom = (viewer.zoom / 1.25).max(0.1);
        }
        SpriteViewerMessage::ZoomReset => {
            viewer.zoom = 1.0;
        }
        SpriteViewerMessage::ZoomToFit => {
            // Zoom to make the frame fill the available preview area
            // Rough fit: use 4× as the max auto-zoom
            viewer.zoom = 4.0;
        }

        // ── Undo / Redo ──────────────────────────────────────────────────────
        SpriteViewerMessage::Undo => {
            viewer.undo();
            if let Some(tab) = app.state.workspace.tabs.iter_mut().find(|t| t.id == tab_id) {
                tab.modified = viewer.dirty;
            }
        }
        SpriteViewerMessage::Redo => {
            viewer.redo();
            if let Some(tab) = app.state.workspace.tabs.iter_mut().find(|t| t.id == tab_id) {
                tab.modified = true;
            }
        }

        // ── Frame editing ────────────────────────────────────────────────────
        SpriteViewerMessage::InsertFrame => {
            viewer.push_undo();
            let seq_idx = viewer.selected_sequence;
            let frame_idx = viewer.selected_frame;
            let new_selected = frame_idx + 1;
            if let Some(ref mut sf) = viewer.sprite_file
                && let Some(seq) = sf.sequences.get_mut(seq_idx)
            {
                let blank = SpriteFrameData {
                    unknown: [0u8; 24],
                    origin_x: 0,
                    origin_y: 0,
                    width: 1,
                    height: 1,
                    raw_pixels: vec![0, 0], // transparent 565 pixel
                };
                seq.frames.insert(frame_idx + 1, blank);
            }
            viewer.selected_frame = new_selected;
            viewer.rebuild_preview_frames();
            viewer.mark_dirty();
            if let Some(tab) = app.state.workspace.tabs.iter_mut().find(|t| t.id == tab_id) {
                tab.modified = true;
            }
        }
        SpriteViewerMessage::DuplicateFrame => {
            viewer.push_undo();
            let seq_idx = viewer.selected_sequence;
            let frame_idx = viewer.selected_frame;
            let new_selected = frame_idx + 1;
            if let Some(ref mut sf) = viewer.sprite_file
                && let Some(seq) = sf.sequences.get_mut(seq_idx)
                && frame_idx < seq.frames.len()
            {
                let dup = seq.frames[frame_idx].clone();
                seq.frames.insert(frame_idx + 1, dup);
            }
            viewer.selected_frame = new_selected;
            viewer.rebuild_preview_frames();
            viewer.mark_dirty();
            if let Some(tab) = app.state.workspace.tabs.iter_mut().find(|t| t.id == tab_id) {
                tab.modified = true;
            }
        }
        SpriteViewerMessage::DeleteFrame => {
            let seq_idx = viewer.selected_sequence;
            let frame_idx = viewer.selected_frame;
            // Check frame count without mutable borrow
            let can_delete = viewer.sprite_file.as_ref().is_some_and(|sf| {
                sf.sequences
                    .get(seq_idx)
                    .is_some_and(|seq| seq.frames.len() > 1)
            });
            if !can_delete {
                viewer.error = Some("Cannot delete the last frame in a sequence".to_string());
                return Task::none();
            }
            viewer.push_undo();
            let new_selected;
            if let Some(ref mut sf) = viewer.sprite_file {
                if let Some(seq) = sf.sequences.get_mut(seq_idx) {
                    seq.frames.remove(frame_idx);
                    new_selected = frame_idx.min(seq.frames.len().saturating_sub(1));
                } else {
                    new_selected = frame_idx;
                }
            } else {
                new_selected = frame_idx;
            }
            viewer.selected_frame = new_selected;
            viewer.rebuild_preview_frames();
            viewer.mark_dirty();
            if let Some(tab) = app.state.workspace.tabs.iter_mut().find(|t| t.id == tab_id) {
                tab.modified = true;
            }
        }
        SpriteViewerMessage::MoveFrameLeft => {
            let seq_idx = viewer.selected_sequence;
            let frame_idx = viewer.selected_frame;
            if frame_idx == 0 {
                return Task::none();
            }
            viewer.push_undo();
            if let Some(ref mut sf) = viewer.sprite_file
                && let Some(seq) = sf.sequences.get_mut(seq_idx)
            {
                seq.frames.swap(frame_idx, frame_idx - 1);
            }
            viewer.selected_frame = frame_idx - 1;
            viewer.rebuild_preview_frames();
            viewer.mark_dirty();
            if let Some(tab) = app.state.workspace.tabs.iter_mut().find(|t| t.id == tab_id) {
                tab.modified = true;
            }
        }
        SpriteViewerMessage::MoveFrameRight => {
            let seq_idx = viewer.selected_sequence;
            let frame_idx = viewer.selected_frame;
            // Check bounds without mutable borrow
            let can_move = viewer.sprite_file.as_ref().is_some_and(|sf| {
                sf.sequences
                    .get(seq_idx)
                    .is_some_and(|seq| frame_idx + 1 < seq.frames.len())
            });
            if !can_move {
                return Task::none();
            }
            viewer.push_undo();
            if let Some(ref mut sf) = viewer.sprite_file
                && let Some(seq) = sf.sequences.get_mut(seq_idx)
            {
                seq.frames.swap(frame_idx, frame_idx + 1);
            }
            viewer.selected_frame = frame_idx + 1;
            viewer.rebuild_preview_frames();
            viewer.mark_dirty();
            if let Some(tab) = app.state.workspace.tabs.iter_mut().find(|t| t.id == tab_id) {
                tab.modified = true;
            }
        }
        SpriteViewerMessage::MoveFrameToStart => {
            let seq_idx = viewer.selected_sequence;
            let frame_idx = viewer.selected_frame;
            if frame_idx == 0 {
                return Task::none();
            }
            viewer.push_undo();
            if let Some(ref mut sf) = viewer.sprite_file
                && let Some(seq) = sf.sequences.get_mut(seq_idx)
            {
                let f = seq.frames.remove(frame_idx);
                seq.frames.insert(0, f);
            }
            viewer.selected_frame = 0;
            viewer.rebuild_preview_frames();
            viewer.mark_dirty();
            if let Some(tab) = app.state.workspace.tabs.iter_mut().find(|t| t.id == tab_id) {
                tab.modified = true;
            }
        }
        SpriteViewerMessage::MoveFrameToEnd => {
            let seq_idx = viewer.selected_sequence;
            let frame_idx = viewer.selected_frame;
            // Check bounds without mutable borrow
            let can_move = viewer.sprite_file.as_ref().is_some_and(|sf| {
                sf.sequences
                    .get(seq_idx)
                    .is_some_and(|seq| frame_idx + 1 < seq.frames.len())
            });
            if !can_move {
                return Task::none();
            }
            viewer.push_undo();
            if let Some(ref mut sf) = viewer.sprite_file
                && let Some(seq) = sf.sequences.get_mut(seq_idx)
            {
                let f = seq.frames.remove(frame_idx);
                seq.frames.push(f);
            }
            // After moving the frame to the end, select the last position.
            // Use current frame_counts which was accurate before the edit.
            viewer.selected_frame = viewer
                .frame_counts
                .get(seq_idx)
                .copied()
                .unwrap_or(0)
                .saturating_sub(1);
            viewer.rebuild_preview_frames();
            viewer.mark_dirty();
            if let Some(tab) = app.state.workspace.tabs.iter_mut().find(|t| t.id == tab_id) {
                tab.modified = true;
            }
        }

        // ── PNG import ──────────────────────────────────────────────────────
        SpriteViewerMessage::ImportPngFrame | SpriteViewerMessage::ImportPngReplace => {
            let _is_replace = matches!(message, SpriteViewerMessage::ImportPngReplace);
            return Task::perform(
                async move {
                    let file = rfd::AsyncFileDialog::new()
                        .add_filter("PNG", &["png"])
                        .pick_file()
                        .await;
                    let Some(file) = file else {
                        return SpriteViewerMessage::PngImportReady(Err("Cancelled".to_string()));
                    };
                    let bytes = file.read().await;
                    match decode_png_to_rgba(&bytes) {
                        Ok((rgba, w, h)) => SpriteViewerMessage::PngImportReady(Ok((rgba, w, h))),
                        Err(e) => SpriteViewerMessage::PngImportReady(Err(e)),
                    }
                },
                |msg| {
                    crate::message::Message::Editor(crate::message::EditorMessage::SpriteViewer(
                        msg,
                    ))
                },
            );
        }
        SpriteViewerMessage::PngImportReady(result) => {
            let (rgba, w, h) = match result {
                Ok(data) => data,
                Err(e) => {
                    if e != "Cancelled" {
                        viewer.error = Some(e);
                    }
                    return Task::none();
                }
            };
            viewer.push_undo();
            // Build a new frame from the decoded PNG
            let pixel_count = (w * h) as usize;
            let raw_size = pixel_count * 2;
            let mut raw_pixels = vec![0u8; raw_size];
            for i in 0..pixel_count.min(rgba.len() / 4) {
                let rbase = i * 4;
                let rgb565 = dispel_core::sprite::rgba_to_rgb565_bytes(
                    rgba[rbase],
                    rgba[rbase + 1],
                    rgba[rbase + 2],
                    rgba[rbase + 3],
                );
                let base = i * 2;
                raw_pixels[base] = rgb565[0];
                raw_pixels[base + 1] = rgb565[1];
            }
            let new_frame = SpriteFrameData {
                unknown: [0u8; 24],
                origin_x: 0,
                origin_y: 0,
                width: w as i32,
                height: h as i32,
                raw_pixels,
            };

            let seq_idx = viewer.selected_sequence;
            let frame_idx = viewer.selected_frame;
            let new_selected = frame_idx + 1;
            if let Some(ref mut sf) = viewer.sprite_file
                && let Some(seq) = sf.sequences.get_mut(seq_idx)
            {
                seq.frames.insert(frame_idx + 1, new_frame);
            }
            viewer.selected_frame = new_selected;
            viewer.rebuild_preview_frames();
            viewer.mark_dirty();
            if let Some(tab) = app.state.workspace.tabs.iter_mut().find(|t| t.id == tab_id) {
                tab.modified = true;
            }
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
            let frames: Vec<(usize, u32, u32, Vec<u8>)> = viewer
                .frames
                .iter()
                .map(|f| (f.frame_idx, f.width, f.height, f.rgba_bytes.clone()))
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

// ── Save logic ────────────────────────────────────────────────────────────────

async fn save_sprite_file(
    path: PathBuf,
    sprite_file: Option<dispel_core::sprite::SpriteFile>,
    recording: Option<crate::editors::sprite_editor::message::RecordingParams>,
) -> Result<String, String> {
    let sf = sprite_file.ok_or_else(|| "No sprite data loaded".to_string())?;
    tokio::task::spawn_blocking(move || {
        write_sprite_to_path(&path, &sf).map_err(|e| e.to_string())?;

        if let Some(params) = recording {
            let current_bytes =
                std::fs::read(&path).map_err(|e| format!("Failed to read saved file: {e}"))?;
            crate::editors::mod_packager::recording::record_file_replace(
                &params.workspace_root,
                &params.game_path,
                &params.mod_slug,
                &params.relative_path,
                &current_bytes,
            )
            .map_err(|e| format!("Recording error: {e}"))?;
        }

        Ok(format!("Saved → {}", path.display()))
    })
    .await
    .map_err(|e| e.to_string())?
}

// ── Export logic ──────────────────────────────────────────────────────────────

async fn perform_export(
    format: ExportFormat,
    frames: Vec<(usize, u32, u32, Vec<u8>)>,
    sprite_name: String,
    export_dir: PathBuf,
) -> Result<String, String> {
    match format {
        ExportFormat::PngFrames => export_png_frames(&frames, &sprite_name, &export_dir),
        ExportFormat::SpriteSheet => export_sprite_sheet(&frames, &sprite_name, &export_dir),
    }
}

fn export_png_frames(
    frames: &[(usize, u32, u32, Vec<u8>)],
    sprite_name: &str,
    export_dir: &Path,
) -> Result<String, String> {
    use image::ImageEncoder;
    use image::codecs::png::PngEncoder;

    let dir = export_dir.join(sprite_name);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    for (idx, width, height, rgba) in frames {
        let path = dir.join(format!("frame_{:03}.png", idx));
        let mut png_buf = Vec::new();
        let encoder = PngEncoder::new(&mut png_buf);
        encoder
            .write_image(rgba, *width, *height, image::ExtendedColorType::Rgba8)
            .map_err(|e| e.to_string())?;
        std::fs::write(&path, &png_buf).map_err(|e| e.to_string())?;
    }
    Ok(format!("Saved {} frames → {}", frames.len(), dir.display()))
}

fn export_sprite_sheet(
    frames: &[(usize, u32, u32, Vec<u8>)],
    sprite_name: &str,
    export_dir: &Path,
) -> Result<String, String> {
    if frames.is_empty() {
        return Err("No frames to export".to_string());
    }

    use image::RgbaImage;

    let total_width: u32 = frames.iter().map(|(_, w, _, _)| *w).sum();
    let max_height: u32 = frames.iter().map(|(_, _, h, _)| *h).max().unwrap_or(0);

    let mut sheet = RgbaImage::new(total_width, max_height);
    let mut x_offset = 0u32;
    for (_, w, h, rgba) in frames {
        if *w == 0 || *h == 0 {
            continue;
        }
        let img = RgbaImage::from_raw(*w, *h, rgba.clone())
            .ok_or_else(|| format!("Invalid frame dimensions: {}×{}", w, h))?;
        for (x, y, px) in img.enumerate_pixels() {
            if x + x_offset < total_width && y < max_height {
                sheet.put_pixel(x + x_offset, y, *px);
            }
        }
        x_offset += w;
    }

    let path = export_dir.join(format!("{}_sheet.png", sprite_name));
    sheet.save(&path).map_err(|e| e.to_string())?;
    Ok(format!("Saved sprite sheet → {}", path.display()))
}

// ── PNG decode helper ─────────────────────────────────────────────────────────

fn decode_png_to_rgba(bytes: &[u8]) -> Result<(Vec<u8>, u32, u32), String> {
    let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Ok((rgba.into_raw(), w, h))
}
