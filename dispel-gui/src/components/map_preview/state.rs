//! State types for the read-only map preview component.

use iced::widget::canvas;
use iced::widget::image::Handle;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

// ── Loading state ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapPreviewLoading {
    Idle,
    Loading,
    Loaded,
    Failed(String),
}

impl Default for MapPreviewLoading {
    fn default() -> Self {
        Self::Idle
    }
}

// ── Entity marker ─────────────────────────────────────────────────────────────

/// A single entity position on the map, derived from save file data.
#[derive(Debug, Clone)]
pub struct PreviewEntity {
    /// Tile coordinates (same grid as map editor).
    pub tile_x: i32,
    pub tile_y: i32,
    pub kind: EntityKind,
    /// Human-readable label (e.g. monster name, NPC name, item name).
    pub label: String,
    /// True when the coordinate mapping is confirmed (draw items, NPCs).
    /// False when the mapping is speculative (monsters, extras).
    pub confirmed: bool,
    /// Entity DB ID for sprite lookup (monster_db_id for monsters,
    /// npc_ini_id for NPCs, extra_ini_id for Extra objects).
    /// None for draw items (no sprite).
    pub db_id: Option<i32>,
}

/// Category of entity marker on the preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    Monster,
    Npc,
    Extra,
    DrawItem,
}

// ── Layer toggles ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewLayer {
    Ground,
    Buildings,
    Roofs,
    InternalSprites,
    Monsters,
    Npcs,
    Extras,
    DrawItems,
}

// ── View state ────────────────────────────────────────────────────────────────

/// Viewport and visibility settings for the preview canvas.
#[derive(Debug)]
pub struct MapPreviewViewState {
    /// Pixel pan offset.
    pub pan_x: f32,
    pub pan_y: f32,
    /// Zoom factor (1.0 = 1:1 pixel).
    pub zoom: f32,
    // Layer visibility toggles
    pub show_ground: bool,
    pub show_buildings: bool,
    pub show_roofs: bool,
    pub show_internal_sprites: bool,
    pub show_monsters: bool,
    pub show_npcs: bool,
    pub show_extras: bool,
    pub show_draw_items: bool,
    /// Last known canvas size, used by FitToWindow.
    pub last_canvas_w: f32,
    pub last_canvas_h: f32,
    /// Cached tile-layer frame (avoids redraw on every frame).
    pub tile_cache: canvas::Cache,
}

impl Clone for MapPreviewViewState {
    fn clone(&self) -> Self {
        Self {
            pan_x: self.pan_x,
            pan_y: self.pan_y,
            zoom: self.zoom,
            show_ground: self.show_ground,
            show_buildings: self.show_buildings,
            show_roofs: self.show_roofs,
            show_internal_sprites: self.show_internal_sprites,
            show_monsters: self.show_monsters,
            show_npcs: self.show_npcs,
            show_extras: self.show_extras,
            show_draw_items: self.show_draw_items,
            last_canvas_w: self.last_canvas_w,
            last_canvas_h: self.last_canvas_h,
            tile_cache: canvas::Cache::new(),
        }
    }
}

impl Default for MapPreviewViewState {
    fn default() -> Self {
        Self {
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
            show_ground: true,
            show_buildings: true,
            show_roofs: true,
            show_internal_sprites: true,
            show_monsters: true,
            show_npcs: true,
            show_extras: true,
            show_draw_items: true,
            last_canvas_w: 800.0,
            last_canvas_h: 600.0,
            tile_cache: canvas::Cache::new(),
        }
    }
}

// ── Decoded sprite frame for preview rendering ────────────────────────────────

/// A single decoded sprite frame (always frame[0]) for use in the map preview.
#[derive(Debug, Clone)]
pub struct PreviewSprite {
    pub handle: Handle,
    pub width: u32,
    pub height: u32,
    pub origin_x: i32,
    pub origin_y: i32,
}

/// A decoded internal sprite from the .map file (thrones, decor, vases …).
#[derive(Debug, Clone)]
pub struct PreviewInternalSprite {
    /// Iced image handle with decoded RGBA pixels.
    pub handle: Handle,
    /// X position in occluded pixel space (block.sprite_x + nox).
    pub x: i32,
    /// Y position in occluded pixel space (block.sprite_y + noy).
    pub y: i32,
    /// Depth sort key (block.sprite_bottom_right_y).
    pub sort_y: i32,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
}

// ── Preview state ─────────────────────────────────────────────────────────────

/// Full state for one map preview instance.
pub struct MapPreviewState {
    /// Loading progress.
    pub loading: MapPreviewLoading,
    /// Loaded map data (set after map file parse).
    pub map_data: Option<Arc<dispel_core::map::MapData>>,
    /// Decoded ground tile image handles (key = tile_id).
    pub gtl_handles: HashMap<i32, Handle>,
    /// Decoded building tile image handles (key = tile_id).
    pub btl_handles: HashMap<i32, Handle>,
    /// True once tileset pixel decoding is complete.
    pub tiles_ready: bool,
    /// The diagonal = tiled_map_width + tiled_map_height (cached for rendering).
    pub diagonal: i32,
    /// Viewport state (pan, zoom, layers).
    pub view: MapPreviewViewState,
    /// Precomputed entity marker positions from save file data.
    pub entity_markers: Vec<PreviewEntity>,
    /// Game path for the workspace (needed for loading).
    pub game_path: Option<PathBuf>,
    /// The map filename stem (e.g. "cat1") for the current preview.
    pub map_stem: Option<String>,
    /// Decoded entity sprites parallel to entity_markers (None when sprite not
    /// found or entity type doesn't support sprites).
    pub entity_sprites: Vec<Option<PreviewSprite>>,
    /// True once async sprite loading has completed.
    pub sprites_ready: bool,
    /// Decoded internal map sprites (thrones, decor, vases …).
    pub internal_sprites: Vec<PreviewInternalSprite>,
    /// Index into `entity_markers` of the clicked/inspected entity, if any.
    pub selected_marker: Option<usize>,
}

impl Default for MapPreviewState {
    fn default() -> Self {
        Self {
            loading: MapPreviewLoading::Idle,
            map_data: None,
            gtl_handles: HashMap::new(),
            btl_handles: HashMap::new(),
            tiles_ready: false,
            diagonal: 0,
            view: MapPreviewViewState::default(),
            entity_markers: Vec::new(),
            game_path: None,
            map_stem: None,
            entity_sprites: Vec::new(),
            sprites_ready: false,
            internal_sprites: Vec::new(),
            selected_marker: None,
        }
    }
}

impl MapPreviewState {
    /// Whether preview data is fully loaded and ready to render.
    /// Only checks tiles — sprite loading is optional (sprites are a bonus).
    pub fn is_ready(&self) -> bool {
        self.loading == MapPreviewLoading::Loaded && self.tiles_ready
    }
}
