use super::message::{MapDataHandle, MapViewMode, SelectedEntity};
use crate::components::loading_state::LoadingState;
use iced::widget::canvas;
use iced::widget::image::Handle;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;

const MAX_MAP_HISTORY: usize = 100;

// ── Sprite export dialog state ────────────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq)]
pub enum SpriteExportStatus {
    #[default]
    Idle,
    Exporting,
    Done(String),
    Error(String),
}

#[derive(Debug, Clone, Default)]
pub struct SpriteExportDialogState {
    pub export_dir: Option<PathBuf>,
    pub status: SpriteExportStatus,
}

// ── Sub-types ─────────────────────────────────────────────────────────────────

/// One decoded thumbnail per unique sprite sequence for the Sprites browser.
pub struct SpriteSequenceHandle {
    pub sequence_idx: usize,
    pub handle: Handle,
    pub width: u32,
    pub height: u32,
    pub placement_count: usize,
    pub placements: Vec<(i32, i32)>,
}

/// A decoded internal-map sprite ready to draw on the canvas.
pub struct InternalSpriteHandle {
    /// Pixel x in the full (non-occluded) image space.
    pub x: i32,
    /// Pixel y in the full (non-occluded) image space.
    pub y: i32,
    /// Y-sort key for interlaced rendering (`sprite_bottom_right_y` from the file).
    pub sort_y: i32,
    pub handle: Handle,
    pub width: u32,
    pub height: u32,
}

/// A decoded entity sprite (NPC / monster / extra) ready to draw.
pub struct EntitySpriteHandle {
    pub handle: Handle,
    pub width: u32,
    pub height: u32,
    pub origin_x: i32,
    pub origin_y: i32,
    pub flip: bool,
}

/// A single recorded change for map-editor undo/redo.
#[derive(Clone, Debug)]
pub struct MapEditAction {
    pub entity: SelectedEntity,
    pub field: String,
    pub old_value: String,
    pub new_value: String,
}

// ── Dialog preview state ───────────────────────────────────────────────────────

/// Loaded dialog data for the NPC dialog preview modal.
pub struct DialogPreviewState {
    pub npc_index: usize,
    pub dialog_scripts: Vec<dispel_core::references::dialogue_script::DialogueScript>,
    pub dialog_paragraphs: Vec<dispel_core::references::dialogue_paragraph::DialogueParagraph>,
}

// ── MapViewState ──────────────────────────────────────────────────────────────

/// Viewport, layer-visibility, and cursor state for the map canvas.
///
/// Contains everything that changes during interactive use (pan, zoom, cursor,
/// selection) but does *not* hold loaded data or persistence state.
pub struct MapViewState {
    /// Pixel pan offset (canvas translation).
    pub pan_x: f32,
    pub pan_y: f32,
    /// Zoom factor (1.0 = 1:1 pixel).
    pub zoom: f32,
    // Layer visibility toggles
    pub show_ground: bool,
    pub show_buildings: bool,
    pub show_roofs: bool,
    pub show_internal_sprites: bool,
    pub show_collisions: bool,
    pub show_events: bool,
    pub show_monsters: bool,
    pub show_npcs: bool,
    pub show_npc_waypoints: bool,
    pub show_objects: bool,
    /// Last known cursor position in canvas-local pixel coordinates.
    /// Set to f32::NAN when the cursor is not over the canvas.
    pub cursor_canvas_x: f32,
    pub cursor_canvas_y: f32,
    /// Last observed canvas size (updated from `MouseMoved` events).
    /// Used by `FitToWindow` to compute the correct zoom / pan.
    pub last_canvas_w: f32,
    pub last_canvas_h: f32,
    /// Which top-level view is shown (map canvas or sprite browser).
    pub view_mode: MapViewMode,
    /// Selected sprite sequence index in the Sprites browser.
    pub selected_sprite_sequence: Option<usize>,
    /// Currently selected entity in the inspector panel.
    pub selected_entity: Option<SelectedEntity>,
    /// Cached tile-layer frame. Clear whenever pan, zoom, tiles, or entity
    /// sprites change. Avoids redrawing the expensive tile layer on every
    /// cursor-move event (which only affects the overlay canvas).
    pub tile_layer_cache: canvas::Cache,
    /// Cached static overlay frame (collisions, events, selection ring).
    /// Separate from `tile_layer_cache` so that cursor moves don't invalidate
    /// the collision/event geometry.  Clear on pan, zoom, layer toggle, or
    /// selection change — but NOT on `MouseMoved`.
    pub overlay_cache: canvas::Cache,
    /// NPC dialog preview modal state (None = closed).
    pub dialog_preview: Option<DialogPreviewState>,
}

impl Default for MapViewState {
    fn default() -> Self {
        Self {
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
            show_ground: true,
            show_buildings: true,
            show_roofs: true,
            show_internal_sprites: true,
            show_collisions: false,
            show_events: false,
            show_monsters: true,
            show_npcs: true,
            show_npc_waypoints: false,
            show_objects: true,
            cursor_canvas_x: f32::NAN,
            cursor_canvas_y: f32::NAN,
            last_canvas_w: 1200.0,
            last_canvas_h: 800.0,
            view_mode: MapViewMode::Map,
            selected_sprite_sequence: None,
            selected_entity: None,
            tile_layer_cache: canvas::Cache::new(),
            overlay_cache: canvas::Cache::new(),
            dialog_preview: None,
        }
    }
}

// ── MapDataState ──────────────────────────────────────────────────────────────

/// Loaded map data, entity lists, file paths, and edit history.
///
/// Contains everything that is loaded from disk or mutated by user edits.
/// Separated from `MapViewState` so that viewport changes (pan, zoom, cursor)
/// don't require reasoning about data-lifecycle concerns, and vice-versa.
pub struct MapDataState {
    pub map_path: Option<PathBuf>,
    pub loading_state: LoadingState<MapDataHandle>,
    /// Decoded tile image handles for ground (GTL) tiles.
    pub gtl_handles: HashMap<i32, Handle>,
    /// Decoded tile image handles for building (BTL) tiles.
    pub btl_handles: HashMap<i32, Handle>,
    /// True once tile pixel data has been decoded and handles are ready.
    pub tiles_ready: bool,
    /// Internal-map sprites (thrones, pillars, etc.) decoded from the .map file.
    pub internal_sprite_handles: Vec<InternalSpriteHandle>,
    /// Per-sequence thumbnails for the Sprites browser (one per unique sequence).
    pub sprite_sequence_handles: Vec<SpriteSequenceHandle>,
    // Entity overlays (loaded from adjacent .ref files)
    pub monsters: Vec<dispel_core::MonsterRef>,
    pub npcs: Vec<dispel_core::NPC>,
    pub extra_refs: Vec<dispel_core::ExtraRef>,
    /// Per-entity sprite handle (parallel to the entity vecs).
    pub monster_sprites: Vec<Option<EntitySpriteHandle>>,
    pub npc_sprites: Vec<Option<EntitySpriteHandle>>,
    pub extra_sprites: Vec<Option<EntitySpriteHandle>>,
    /// NPC ID → sprite filename lookup (from Npc.ini), for re-resolving sprites
    /// when the looking_direction field changes.
    pub npc_id_to_sprite: HashMap<i32, String>,
    /// Resolved paths to entity .ref files (for save-back).
    pub monster_ref_path: Option<PathBuf>,
    pub npc_ref_path: Option<PathBuf>,
    pub extra_ref_path: Option<PathBuf>,
    /// Resolved paths to GTL/BTL tileset files (for PNG export).
    pub gtl_path: Option<PathBuf>,
    pub btl_path: Option<PathBuf>,
    /// Undo/redo stacks for entity field edits.
    pub undo_stack: VecDeque<MapEditAction>,
    pub redo_stack: VecDeque<MapEditAction>,
    /// True when there are unsaved entity changes.
    pub dirty: bool,
    /// True while an async entity save is in flight.
    pub is_saving: bool,
    /// True while an async PNG export is in flight.
    pub is_exporting: bool,
    /// Last save/export status message for display in the toolbar.
    pub status_msg: Option<String>,
    /// Sprite export dialog state (None = dialog closed).
    pub sprite_export_dialog: Option<SpriteExportDialogState>,
}

impl Default for MapDataState {
    fn default() -> Self {
        Self {
            map_path: None,
            loading_state: LoadingState::Idle,
            gtl_handles: HashMap::new(),
            btl_handles: HashMap::new(),
            tiles_ready: false,
            internal_sprite_handles: Vec::new(),
            sprite_sequence_handles: Vec::new(),
            monsters: Vec::new(),
            npcs: Vec::new(),
            extra_refs: Vec::new(),
            monster_sprites: Vec::new(),
            npc_sprites: Vec::new(),
            extra_sprites: Vec::new(),
            npc_id_to_sprite: HashMap::new(),
            monster_ref_path: None,
            npc_ref_path: None,
            extra_ref_path: None,
            gtl_path: None,
            btl_path: None,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            dirty: false,
            is_saving: false,
            is_exporting: false,
            status_msg: None,
            sprite_export_dialog: None,
        }
    }
}

impl MapDataState {
    pub fn map_data(&self) -> Option<&MapDataHandle> {
        self.loading_state.data()
    }

    /// Recompute the sprite for NPC at `idx` based on its current `looking_direction`.
    ///
    /// Called after a direction field change so the canvas displays the new
    /// orientation without requiring a full entity reload.
    pub fn recompute_npc_sprite(&mut self, idx: usize, game_path: &std::path::Path) {
        use dispel_core::map::sprite_loader::load_sprite_frames;

        let Some(npc) = self.npcs.get(idx) else {
            return;
        };
        let Some(sprite_name) = self.npc_id_to_sprite.get(&npc.npc_id) else {
            return;
        };

        // Direction → (sequence_index, flip) — same logic as load_entities().
        let dir = i32::from(npc.looking_direction);
        let (seq, flip) = if dir > 4 {
            ((8 - dir) as usize, true)
        } else {
            (dir as usize, false)
        };

        // Case-insensitive file resolution (mirrors the `resolve` closure in load_entities).
        let sub_dir = game_path.join("NpcInGame");
        let spr_path = [
            sprite_name.clone(),
            sprite_name.to_ascii_uppercase(),
            sprite_name.to_ascii_lowercase(),
        ]
        .into_iter()
        .find_map(|n| {
            let p = sub_dir.join(&n);
            p.exists().then_some(p)
        })
        .unwrap_or_else(|| sub_dir.join(sprite_name));

        let sprite_handle = load_sprite_frames(&spr_path).and_then(|frames| {
            frames.get(seq).or_else(|| frames.first()).map(|frame| {
                let w = frame.image.width();
                let h = frame.image.height();
                EntitySpriteHandle {
                    handle: Handle::from_rgba(w, h, frame.image.as_raw().to_vec()),
                    width: w,
                    height: h,
                    origin_x: frame.origin_x,
                    origin_y: frame.origin_y,
                    flip,
                }
            })
        });

        if let Some(handle) = sprite_handle {
            self.npc_sprites[idx] = Some(handle);
        }
    }
}

// ── MapEditorState ────────────────────────────────────────────────────────────

/// Top-level state for one map editor tab.
///
/// Composes `MapViewState` (viewport / interaction) and `MapDataState`
/// (loaded data / persistence) so each concern can be reasoned about
/// independently.
#[derive(Default)]
pub struct MapEditorState {
    pub view: MapViewState,
    pub data: MapDataState,
}

impl MapEditorState {
    /// Convenience delegate: whether map data is loaded.
    pub fn map_data(&self) -> Option<&MapDataHandle> {
        self.data.map_data()
    }

    /// Push a reversible field-change action onto the undo stack and mark dirty.
    pub fn push_undo(&mut self, action: MapEditAction) {
        self.data.undo_stack.push_front(action);
        if self.data.undo_stack.len() > MAX_MAP_HISTORY {
            self.data.undo_stack.pop_back();
        }
        self.data.redo_stack.clear();
        self.data.dirty = true;
    }

    /// Pop the most-recent undo action. Pushes the inverted action to redo.
    /// Returns the action with `old_value`/`new_value` as recorded (so the
    /// caller applies `old_value` to revert).
    pub fn pop_undo(&mut self) -> Option<MapEditAction> {
        let action = self.data.undo_stack.pop_front()?;
        let redo_entry = MapEditAction {
            entity: action.entity,
            field: action.field.clone(),
            old_value: action.old_value.clone(),
            new_value: action.new_value.clone(),
        };
        self.data.redo_stack.push_front(redo_entry);
        Some(action)
    }

    /// Pop the most-recent redo action. Pushes the action back to undo.
    /// Returns the action; caller applies `new_value` to re-apply the change.
    pub fn pop_redo(&mut self) -> Option<MapEditAction> {
        let action = self.data.redo_stack.pop_front()?;
        let undo_entry = MapEditAction {
            entity: action.entity,
            field: action.field.clone(),
            old_value: action.old_value.clone(),
            new_value: action.new_value.clone(),
        };
        self.data.undo_stack.push_front(undo_entry);
        Some(action)
    }
}
