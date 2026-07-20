//! State types for the read-only map preview component.

use crate::components::map_render::{EntitySpriteHandle, InternalSpriteHandle, MapViewState};
use crate::editors::map_editor::message::MapDataHandle;
use iced::widget::image::Handle;
use std::collections::HashMap;
use std::path::PathBuf;

// ── Loading state ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum MapPreviewLoading {
    #[default]
    Idle,
    Loading,
    Loaded,
    Failed(String),
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
    /// True when the entity is dead (e.g. monster with hp_current == 0).
    /// Dead monsters render using the death animation sequence.
    pub is_dead: bool,
    /// NPC looking direction (1–8) used to select the sprite sequence + flip,
    /// mirroring the map editor's NPC facing logic.  0 for non-NPC entities.
    pub look_direction: u8,
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

// ── Preview state ─────────────────────────────────────────────────────────────

/// Full state for one map preview instance.
pub struct MapPreviewState {
    /// Loading progress.
    pub loading: MapPreviewLoading,
    /// Loaded map data (set after map file parse).
    pub map_data: Option<MapDataHandle>,
    /// Decoded ground tile image handles (key = tile_id).
    pub gtl_handles: HashMap<i32, Handle>,
    /// Decoded building tile image handles (key = tile_id).
    pub btl_handles: HashMap<i32, Handle>,
    /// True once tileset pixel decoding is complete.
    pub tiles_ready: bool,
    /// The diagonal = tiled_map_width + tiled_map_height (cached for rendering).
    pub diagonal: i32,
    /// Viewport state (pan, zoom, layers).
    pub view: MapViewState,
    /// Precomputed entity marker positions from save file data.
    pub entity_markers: Vec<PreviewEntity>,
    /// Game path for the workspace (needed for loading).
    pub game_path: Option<PathBuf>,
    /// The map filename stem (e.g. "cat1") for the current preview.
    pub map_stem: Option<String>,
    /// Decoded entity sprites parallel to entity_markers (None when sprite not
    /// found or entity type doesn't support sprites).
    pub entity_sprites: Vec<Option<EntitySpriteHandle>>,
    /// True once async sprite loading has completed.
    pub sprites_ready: bool,
    /// Decoded internal map sprites (thrones, decor, vases …).
    pub internal_sprites: Vec<InternalSpriteHandle>,
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
            view: MapViewState::default(),
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

// ── MapRenderSource implementation ────────────────────────────────────────────

use crate::components::map_render::traits::{
    EntityKind as RenderEntityKind, EntityRenderData, MapRenderSource,
};

impl MapRenderSource for MapPreviewState {
    fn map_data(&self) -> Option<&MapDataHandle> {
        self.map_data.as_ref()
    }

    fn gtl_handles(&self) -> &HashMap<i32, Handle> {
        &self.gtl_handles
    }

    fn btl_handles(&self) -> &HashMap<i32, Handle> {
        &self.btl_handles
    }

    fn tiles_ready(&self) -> bool {
        self.tiles_ready
    }

    fn view(&self) -> &MapViewState {
        &self.view
    }

    fn internal_sprite_handles(&self) -> &[InternalSpriteHandle] {
        &self.internal_sprites
    }

    fn entity_count(&self) -> usize {
        self.entity_markers.len()
    }

    fn entity_data(&self, idx: usize) -> Option<EntityRenderData<'_>> {
        let marker = self.entity_markers.get(idx)?;
        let map_handle = self.map_data.as_ref()?;
        let model = &map_handle.0.model;
        let noy = model.map_non_occluded_start_y;

        let sort_key = {
            let img_y = dispel_core::map::types::convert_map_coords_to_image_coords(
                marker.tile_x,
                marker.tile_y,
                self.diagonal,
            )
            .1;
            img_y + 32 - noy
        };

        let kind = match marker.kind {
            EntityKind::Monster => RenderEntityKind::Monster,
            EntityKind::Npc => RenderEntityKind::Npc,
            EntityKind::Extra => RenderEntityKind::Extra,
            EntityKind::DrawItem => RenderEntityKind::DrawItem,
        };

        let visible = match marker.kind {
            EntityKind::Monster => self.view.show_monsters,
            EntityKind::Npc => self.view.show_npcs,
            EntityKind::Extra => self.view.show_objects,
            EntityKind::DrawItem => self.view.show_draw_items,
        };

        Some(EntityRenderData {
            tile_x: marker.tile_x,
            tile_y: marker.tile_y,
            sort_key,
            sprite: self.entity_sprites.get(idx).and_then(|o| o.as_ref()),
            kind,
            visible,
        })
    }
}
