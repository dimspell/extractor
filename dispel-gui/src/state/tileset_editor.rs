use crate::message::editor::tileset::TileExportFormat;
use dispel_core::map::tileset::{extract, TILE_HEIGHT, TILE_WIDTH};
use dispel_core::sprite::Color;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum TilesetFileType {
    Gtl,
    Btl,
}

#[derive(Debug, Clone)]
pub struct TileHandle {
    pub image: iced::widget::image::Handle,
}

// ── Export dialog state ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ExportStatus {
    #[default]
    Idle,
    Exporting,
    Done(String),
    Error(String),
}

#[derive(Debug, Clone, Default)]
pub struct ExportDialogState {
    pub format: TileExportFormat,
    pub export_dir: Option<PathBuf>,
    pub status: ExportStatus,
}

// ── Editor state ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TilesetEditorState {
    pub path: PathBuf,
    pub name: String,
    pub file_type: TilesetFileType,
    pub tiles: Vec<TileHandle>,
    pub zoom: f32,
    pub error: Option<String>,
    pub export_dialog: Option<ExportDialogState>,
}

impl TilesetEditorState {
    pub fn load(path: &Path) -> Self {
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let file_type = match path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase()
            .as_str()
        {
            "btl" => TilesetFileType::Btl,
            _ => TilesetFileType::Gtl,
        };

        match extract(path) {
            Ok(raw_tiles) => {
                let tiles = raw_tiles
                    .iter()
                    .map(|tile| {
                        let rgba = colors_to_rgba(&tile.colors);
                        TileHandle {
                            image: iced::widget::image::Handle::from_rgba(
                                TILE_WIDTH,
                                TILE_HEIGHT,
                                rgba,
                            ),
                        }
                    })
                    .collect();
                Self {
                    path: path.to_path_buf(),
                    name,
                    file_type,
                    tiles,
                    zoom: 1.0,
                    error: None,
                    export_dialog: None,
                }
            }
            Err(e) => Self {
                path: path.to_path_buf(),
                name,
                file_type,
                tiles: Vec::new(),
                zoom: 1.0,
                error: Some(e.to_string()),
                export_dialog: None,
            },
        }
    }
}

/// Convert a tile's decoded color array to a 62×32 RGBA byte buffer.
/// Uses the same isometric diamond mask as tileset::plot_tile_rgba but without
/// the image crate dependency (avoids version conflicts with dispel_core).
fn colors_to_rgba(colors: &[Color; 1024]) -> Vec<u8> {
    let w = TILE_WIDTH as usize;
    let h = TILE_HEIGHT as usize;
    let mut rgba = vec![0u8; w * h * 4];

    let mask = build_mask();
    let mut src = 0usize;
    for (y, row) in mask.iter().enumerate() {
        let x_off = row[0] as usize;
        let width = row[1] as usize;
        for x in 0..width {
            let c = colors[src];
            src += 1;
            if c.r == 0 && c.g == 0 && c.b == 0 {
                continue;
            }
            let px = x_off + x;
            let dst = (y * w + px) * 4;
            rgba[dst] = c.r;
            rgba[dst + 1] = c.g;
            rgba[dst + 2] = c.b;
            rgba[dst + 3] = 255;
        }
    }
    rgba
}

/// Builds the isometric diamond mask: [[x_offset, width]; 32].
fn build_mask() -> [[i32; 2]; 32] {
    let mut mask = [[0i32; 2]; TILE_HEIGHT as usize];
    let mut pixels_x: i32 = 1;
    let step: i32 = 2;
    let mut direction: i32 = 1;
    let limit = 31;

    for row in mask.iter_mut() {
        row[0] = (TILE_WIDTH as i32) / 2 - pixels_x;
        row[1] = pixels_x * 2;
        pixels_x += step * direction;
        if pixels_x > limit {
            direction = -1;
            pixels_x = limit;
        }
    }
    mask
}
