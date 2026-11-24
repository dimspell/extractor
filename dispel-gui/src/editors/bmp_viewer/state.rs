use iced::Size;
use iced::widget::image::Handle;
use std::path::{Path, PathBuf};

pub const MIN_ZOOM: f32 = 0.5;
pub const MAX_ZOOM: f32 = 4.0;
pub const ZOOM_STEP: f32 = 1.25;

pub struct BmpViewerState {
    pub path: PathBuf,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub handle: Option<Handle>,
    pub error: Option<String>,
    pub zoom: f32,
    pub fit_mode: bool,
}

/// Scale that fits an image inside the available viewport while preserving its
/// aspect ratio. Returns at least a small positive value so a viewport that has
/// not been laid out yet cannot collapse the image to zero.
pub fn fit_scale(viewport: Size, image: (u32, u32)) -> f32 {
    let (w, h) = image;
    if w == 0 || h == 0 || viewport.width <= 0.0 || viewport.height <= 0.0 {
        return 1.0;
    }
    (viewport.width / w as f32)
        .min(viewport.height / h as f32)
        .max(0.01)
}

/// Exact rendered size of the image for the given viewport and zoom settings.
/// Returning a concrete size (instead of `Fill`) lets the surrounding scrollable
/// grow beyond the viewport so zoomed images can be panned.
pub fn rendered_size(viewport: Size, image: (u32, u32), fit_mode: bool, zoom: f32) -> Size {
    let fit = if fit_mode {
        fit_scale(viewport, image)
    } else {
        1.0
    };
    let scale = fit * zoom;
    Size::new(
        (image.0 as f32 * scale).max(1.0),
        (image.1 as f32 * scale).max(1.0),
    )
}

impl BmpViewerState {
    pub fn load_from_path(path: &Path) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let mut state = BmpViewerState {
            path: path.to_path_buf(),
            name,
            width: 0,
            height: 0,
            handle: None,
            error: None,
            zoom: 1.0,
            fit_mode: true,
        };

        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) => {
                state.error = Some(e.to_string());
                return state;
            }
        };

        match image::load_from_memory_with_format(&bytes, image::ImageFormat::Bmp) {
            Ok(decoded) => {
                let rgba = decoded.to_rgba8();
                let (width, height) = rgba.dimensions();
                state.handle = Some(Handle::from_rgba(width, height, rgba.into_raw()));
                state.width = width;
                state.height = height;
            }
            Err(e) => state.error = Some(format!("Invalid BMP image: {e}")),
        }

        state
    }
}
