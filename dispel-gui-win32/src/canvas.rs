//! Custom canvas control for GDI-based rendering.
// Used for map, tileset, and sprite visualization.

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Gdi::*;
use std::path::Path;

/// Canvas rendering modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasMode {
    Tileset,  // Grid of 32x32 tiles (GTL/BTL)
    Sprite,   // Character animation frames (SPR)
    Map,      // Isometric map rendering (MAP)
}

/// Custom canvas control for rendering game assets.
pub struct Canvas {
    pub hwnd: HWND,
    pub mode: CanvasMode,
    pub file_path: Option<PathBuf>,
    pub zoom: f64,
    pub offset_x: i32,
    pub offset_y: i32,
}

impl Canvas {
    pub fn new(parent: HWND) -> Result<Self> {
        unsafe {
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                None,
                WS_CHILD | WS_VISIBLE | SS_OWNERDRAW | SS_NOTIFY,
                0, 0, 0, 0,
                parent,
                None,
                GetModuleHandleW(None)?,
                None,
            );

            Ok(Self {
                hwnd,
                mode: CanvasMode::Tileset,
                file_path: None,
                zoom: 1.0,
                offset_x: 0,
                offset_y: 0,
            })
        }
    }

    /// Load a tileset file (GTL/BTL) for rendering.
    pub fn load_tileset(&mut self, path: &Path) -> Result<()> {
        self.file_path = Some(path.to_path_buf());
        self.mode = CanvasMode::Tileset;
        // TODO: Parse GTL/BTL file and store tile data
        Ok(())
    }

    /// Load a sprite file (SPR) for animation playback.
    pub fn load_sprite(&mut self, path: &Path) -> Result<()> {
        self.file_path = Some(path.to_path_buf());
        self.mode = CanvasMode::Sprite;
        // TODO: Parse SPR file and store sprite frames
        Ok(())
    }

    /// Load a map file (MAP) for isometric rendering.
    pub fn load_map(&mut self, path: &Path) -> Result<()> {
        self.file_path = Some(path.to_path_buf());
        self.mode = CanvasMode::Map;
        // TODO: Parse MAP file and store map data
        Ok(())
    }

    /// Set zoom level.
    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom.clamp(0.1, 10.0);
    }

    /// Set pan offset.
    pub fn set_offset(&mut self, x: i32, y: i32) {
        self.offset_x = x;
        self.offset_y = y;
    }

    /// Render the current content using GDI.
    pub fn render(&self) -> Result<()> {
        unsafe {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(self.hwnd, &mut ps);

            // Get client rect
            let mut rect = RECT::default();
            GetClientRect(self.hwnd, &mut rect);

            // Fill background with dark color
            let hbrush = CreateSolidBrush(RGB(40, 40, 40));
            FillRect(hdc, &rect, hbrush);
            DeleteObject(hbrush as HGDIOBJ);

            // Render based on mode
            match self.mode {
                CanvasMode::Tileset => self.render_tileset(hdc, &rect)?,
                CanvasMode::Sprite => self.render_sprite(hdc, &rect)?,
                CanvasMode::Map => self.render_map(hdc, &rect)?,
            }

            EndPaint(self.hwnd, &ps);
        }

        Ok(())
    }

    fn render_tileset(&self, hdc: HDC, rect: &RECT) -> Result<()> {
        // Draw a placeholder grid for tileset preview
        // Each tile is 32x32 pixels at 1x zoom
        let tile_size = (32.0 * self.zoom) as i32;
        let cols = (rect.right - rect.left) / tile_size + 1;
        let rows = (rect.bottom - rect.top) / tile_size + 1;

        let pen = CreatePen(PS_SOLID, 1, RGB(100, 100, 100));
        let old_pen = SelectObject(hdc, pen as HGDIOBJ);

        for x in 0..cols {
            for y in 0..rows {
                let px = x * tile_size + self.offset_x;
                let py = y * tile_size + self.offset_y;
                Rectangle(hdc, px, py, px + tile_size, py + tile_size);
            }
        }

        SelectObject(hdc, old_pen);
        DeleteObject(pen as HGDIOBJ);

        Ok(())
    }

    fn render_sprite(&self, hdc: HDC, rect: &RECT) -> Result<()> {
        // Draw a placeholder for sprite preview
        let center_x = rect.right / 2 + self.offset_x;
        let center_y = rect.bottom / 2 + self.offset_y;
        let size = (64.0 * self.zoom) as i32;

        let pen = CreatePen(PS_SOLID, 2, RGB(200, 200, 200));
        let old_pen = SelectObject(hdc, pen as HGDIOBJ);
        let brush = CreateSolidBrush(RGB(80, 80, 80));
        let old_brush = SelectObject(hdc, brush as HGDIOBJ);

        Ellipse(hdc, center_x - size / 2, center_y - size / 2, center_x + size / 2, center_y + size / 2);

        SelectObject(hdc, old_brush);
        DeleteObject(brush as HGDIOBJ);
        SelectObject(hdc, old_pen);
        DeleteObject(pen as HGDIOBJ);

        Ok(())
    }

    fn render_map(&self, hdc: HDC, rect: &RECT) -> Result<()> {
        // Draw a placeholder for isometric map preview
        // Isometric projection: x_screen = (x_map - y_map) * tile_width / 2
        //                       y_screen = (x_map + y_map) * tile_height / 2
        let tile_w = 64;
        let tile_h = 32;

        let pen = CreatePen(PS_SOLID, 1, RGB(80, 80, 80));
        let old_pen = SelectObject(hdc, pen as HGDIOBJ);

        // Draw a simple isometric grid
        for i in -10..10 {
            for j in -10..10 {
                let sx = (i - j) * tile_w / 2 + rect.right / 2 + self.offset_x;
                let sy = (i + j) * tile_h / 2 + rect.bottom / 2 + self.offset_y;
                Rectangle(hdc, sx, sy, sx + tile_w, sy + tile_h);
            }
        }

        SelectObject(hdc, old_pen);
        DeleteObject(pen as HGDIOBJ);

        Ok(())
    }
}
