use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Clonable, debuggable handle for the non-Debug `MapData` type.
#[derive(Clone)]
pub struct MapDataHandle(pub Arc<dispel_core::map::MapData>);

impl std::fmt::Debug for MapDataHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapData").finish_non_exhaustive()
    }
}

/// Decoded RGBA pixel data for a set of tiles.
/// Key = tile_id, Value = 62×32×4 RGBA bytes.
#[derive(Clone, Default)]
pub struct TilePixelData {
    pub gtl: HashMap<i32, Vec<u8>>,
    pub btl: HashMap<i32, Vec<u8>>,
}

impl std::fmt::Debug for TilePixelData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TilePixelData")
            .field("gtl_count", &self.gtl.len())
            .field("btl_count", &self.btl.len())
            .finish()
    }
}

/// A single decoded internal-map sprite (throne, pillar, etc.).
/// `x`, `y` are pixel coords relative to the occluded viewport (same space as
/// `SpriteInfoBlock.sprite_x/y`); `pixels` is RGBA row-major.
#[derive(Clone, Debug)]
pub struct DecodedMapSprite {
    pub x: i32,
    pub y: i32,
    /// Y-sort key for interlaced rendering (= `sprite_bottom_right_y` from the map file).
    pub bottom_right_y: i32,
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// A single decoded entity sprite frame (NPC / monster / extra), already
/// at the correct sequence index and including flip state.
#[derive(Clone)]
pub struct DecodedEntitySprite {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub origin_x: i32,
    pub origin_y: i32,
    pub flip: bool,
}

/// Entity data loaded from adjacent .ref files, optionally with decoded sprites.
#[derive(Clone, Default)]
pub struct EntityBundle {
    pub monsters: Vec<dispel_core::MonsterRef>,
    pub npcs: Vec<dispel_core::NPC>,
    pub extra_refs: Vec<dispel_core::ExtraRef>,
    /// Per-monster decoded sprite (parallel to `monsters`; `None` = no sprite found).
    pub monster_sprites: Vec<Option<DecodedEntitySprite>>,
    /// Per-NPC decoded sprite (parallel to `npcs`).
    pub npc_sprites: Vec<Option<DecodedEntitySprite>>,
    /// Per-extra decoded sprite (parallel to `extra_refs`).
    pub extra_sprites: Vec<Option<DecodedEntitySprite>>,
    /// Resolved paths to the source .ref files (for save-back).
    pub monster_ref_path: Option<PathBuf>,
    pub npc_ref_path: Option<PathBuf>,
    pub extra_ref_path: Option<PathBuf>,
}

impl std::fmt::Debug for EntityBundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityBundle")
            .field("monsters", &self.monsters.len())
            .field("npcs", &self.npcs.len())
            .field("extra_refs", &self.extra_refs.len())
            .finish()
    }
}

#[derive(Debug, Clone)]
pub enum MapEditorMessage {
    /// Open a .map file by path.
    Open(usize, PathBuf),
    /// Async map data load + internal sprite decode completed.
    MapLoaded(
        usize,
        Result<(MapDataHandle, Vec<DecodedMapSprite>), String>,
    ),
    /// Async tile decode completed.
    TilesDecoded(usize, Result<TilePixelData, String>),
    /// Canvas panning (pixel offset delta).
    PanChanged(usize, f32, f32),
    /// Canvas zoom change.  First f32 is a *multiplicative factor* (e.g. 1.15 to
    /// zoom in, 0.87 to zoom out).  Last two floats are the canvas-local cursor
    /// position used as the zoom pivot; NaN when zooming from a toolbar button.
    ZoomChanged(usize, f32, f32, f32),
    /// Toggle a display layer.
    LayerToggled(usize, MapLayer),
    /// Fit the viewport to the full map.
    FitToWindow(usize),
    /// Async entity load (monsters/NPCs/extras) completed.
    EntitiesLoaded(usize, EntityBundle),
    /// Canvas cursor position changed (canvas-local pixel coords).
    /// Last two floats are the canvas size at the time of the event; 0.0 when the
    /// cursor left the canvas (used to keep `last_canvas_w/h` up to date).
    MouseMoved(usize, f32, f32, f32, f32),
    /// Left-click on the canvas that didn't result in a drag (canvas-local px).
    CanvasClicked(usize, f32, f32),
    /// Deselect the current entity in the inspector.
    Deselect(usize),
    /// Entity field edited in the inspector.
    EntityFieldChanged(usize, SelectedEntity, String, String),
    /// Save all modified entity .ref files.
    SaveEntities(usize),
    /// Async save completed. Ok carries a status message; Err the error text.
    SaveComplete(usize, Result<String, String>),
    /// Undo the last entity field change.
    Undo(usize),
    /// Redo the last undone entity field change.
    Redo(usize),
    /// Export the full map as a PNG image.
    ExportImage(usize),
    /// Async export completed. Ok carries the output path; Err the error.
    ExportComplete(usize, Result<String, String>),
    /// Clear the status message (auto-dismiss after a delay).
    ClearStatus(usize),
    /// Switch between Map and Sprites view modes.
    SwitchViewMode(usize, MapViewMode),
    /// Select a sprite sequence in the browser (None = deselect).
    SelectSpriteSequence(usize, Option<usize>),
    /// Open the sprite export dialog.
    ShowSpriteExportDialog(usize),
    /// Close the sprite export dialog without exporting.
    CloseSpriteExportDialog(usize),
    /// Open a folder picker to choose the sprite export directory.
    ChooseSpriteExportDir(usize),
    /// Folder picker result (None = cancelled).
    SpriteExportDirChosen(usize, Option<PathBuf>),
    /// Confirm and start the sprite export.
    ConfirmSpriteExport(usize),
    /// Async sprite export completed.
    SpriteExportDone(usize, Result<String, String>),
}

/// Which entity is currently selected in the map editor inspector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedEntity {
    Monster(usize),
    Npc(usize),
    Extra(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapLayer {
    Ground,
    Buildings,
    Roofs,
    InternalSprites,
    Collisions,
    Events,
    Monsters,
    Npcs,
    Objects,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MapViewMode {
    #[default]
    Map,
    Sprites,
}
