use super::message::{MapDataHandle, SelectedEntity};
use crate::components::loading_state::LoadingState;
pub use crate::components::map_render::{EntitySpriteHandle, InternalSpriteHandle, MapViewState};
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
    /// when the waypoint1_facing_direction field changes.
    pub npc_id_to_sprite: HashMap<i32, String>,
    /// Draw items (item placements from Ref/DRAWITEM.ref) for this map.
    pub draw_items: Vec<dispel_core::DrawItem>,
    /// The current map's AllMap.ini ID, used to filter/save draw items.
    /// `None` if the map isn't listed in AllMap.ini (e.g., map file not found in it).
    pub all_map_id: Option<i32>,
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
            draw_items: Vec::new(),
            all_map_id: None,
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

    /// Returns `true` when the `MapDataHandle` Arc is safe to borrow mutably.
    /// During save/export, a clone of the Arc is held by the async task,
    /// making `Arc::get_mut` panic.
    pub fn can_mutate_map_data(&self) -> bool {
        !self.is_saving && !self.is_exporting
    }

    /// Recompute the sprite for NPC at `idx` from its waypoint 1 facing direction.
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
        let dir = i32::from(npc.waypoint1_facing_direction);
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

// ── MapRenderSource implementation ────────────────────────────────────────────

use crate::components::map_render::traits::{EntityKind, EntityRenderData, MapRenderSource};

impl MapRenderSource for MapEditorState {
    fn map_data(&self) -> Option<&MapDataHandle> {
        self.data.map_data()
    }

    fn gtl_handles(&self) -> &HashMap<i32, Handle> {
        &self.data.gtl_handles
    }

    fn btl_handles(&self) -> &HashMap<i32, Handle> {
        &self.data.btl_handles
    }

    fn tiles_ready(&self) -> bool {
        self.data.tiles_ready
    }

    fn view(&self) -> &MapViewState {
        &self.view
    }

    fn internal_sprite_handles(&self) -> &[InternalSpriteHandle] {
        &self.data.internal_sprite_handles
    }

    fn entity_count(&self) -> usize {
        self.data.monsters.len()
            + self.data.npcs.len()
            + self.data.extra_refs.len()
            + self.data.draw_items.len()
    }

    fn entity_data(&self, idx: usize) -> Option<EntityRenderData<'_>> {
        let monster_count = self.data.monsters.len();
        let npc_count = self.data.npcs.len();
        let extra_count = self.data.extra_refs.len();

        let map_handle = self.data.map_data()?;
        let model = &map_handle.0.model;
        let diagonal = model.tiled_map_width + model.tiled_map_height;
        let noy = model.map_non_occluded_start_y;

        let entity_pos = |tx: i32, ty: i32| -> i32 {
            let img_y =
                dispel_core::map::types::convert_map_coords_to_image_coords(tx, ty, diagonal).1;
            img_y + 32 - noy
        };

        if idx < monster_count {
            let m = &self.data.monsters[idx];
            Some(EntityRenderData {
                tile_x: m.map_x,
                tile_y: m.map_y,
                sort_key: entity_pos(m.map_x, m.map_y),
                sprite: self.data.monster_sprites.get(idx)?.as_ref(),
                kind: EntityKind::Monster,
                visible: self.view.show_monsters,
            })
        } else if idx < monster_count + npc_count {
            let npc_idx = idx - monster_count;
            let n = &self.data.npcs[npc_idx];
            let (nx, ny) = crate::editors::map_editor::canvas::hit_test::npc_pos(n);
            Some(EntityRenderData {
                tile_x: nx,
                tile_y: ny,
                sort_key: entity_pos(nx, ny),
                sprite: self.data.npc_sprites.get(npc_idx)?.as_ref(),
                kind: EntityKind::Npc,
                visible: self.view.show_npcs,
            })
        } else if idx < monster_count + npc_count + extra_count {
            let extra_idx = idx - monster_count - npc_count;
            let e = &self.data.extra_refs[extra_idx];
            Some(EntityRenderData {
                tile_x: e.map_x,
                tile_y: e.map_y,
                sort_key: entity_pos(e.map_x, e.map_y),
                sprite: self.data.extra_sprites.get(extra_idx)?.as_ref(),
                kind: EntityKind::Extra,
                visible: self.view.show_objects,
            })
        } else {
            let di_idx = idx - monster_count - npc_count - extra_count;
            let d = &self.data.draw_items[di_idx];
            Some(EntityRenderData {
                tile_x: d.x_coord,
                tile_y: d.y_coord,
                sort_key: entity_pos(d.x_coord, d.y_coord),
                sprite: None,
                kind: EntityKind::DrawItem,
                visible: self.view.show_draw_items,
            })
        }
    }
}
