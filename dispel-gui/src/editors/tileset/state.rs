use crate::editors::tileset::TileExportFormat;
use dispel_core::decode::tile_colors_to_rgba;
use dispel_core::map::tileset::{TILE_HEIGHT, TILE_WIDTH, extract};
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
                        let rgba = tile_colors_to_rgba(&tile.colors);
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

// The 62×32 RGBA conversion and its diamond mask are owned by
// `dispel_core::decode::tile_colors_to_rgba`; the former local copies
// (`colors_to_rgba` / `build_mask`) were removed.
