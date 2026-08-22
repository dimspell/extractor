use crate::editors::map_editor::message::{MapTool, MapViewMode, ObjectBrushMode, SelectedEntity};
use iced::widget::canvas;

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
    /// Observed shadow/lighting pass (fog-of-war on Dark maps).
    pub show_shadows: bool,
    pub show_internal_sprites: bool,
    pub show_collisions: bool,
    pub show_events: bool,
    pub show_monsters: bool,
    pub show_npcs: bool,
    pub show_npc_waypoints: bool,
    pub show_objects: bool,
    pub show_draw_items: bool,
    pub show_object_ids: bool,
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
    /// Active editing tool. Editing a layer requires selecting its tool;
    /// layer visibility alone no longer enables editing.
    pub active_tool: MapTool,
    /// How the object-id brush is applied when the ObjectId tool clicks a tile.
    pub object_brush_mode: ObjectBrushMode,
    /// Whether the layers dropdown popover is open.
    pub layers_popover_open: bool,
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
    pub dialog_preview: Option<crate::editors::map_editor::state::DialogPreviewState>,
    /// Interactive conversation display state (None = not in conversation).
    pub conversation: Option<crate::editors::map_editor::state::ConversationState>,
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
            show_shadows: true,
            show_internal_sprites: true,
            show_collisions: false,
            show_events: false,
            show_monsters: true,
            show_npcs: true,
            show_npc_waypoints: false,
            show_objects: true,
            show_draw_items: true,
            show_object_ids: false,
            cursor_canvas_x: f32::NAN,
            cursor_canvas_y: f32::NAN,
            last_canvas_w: 1200.0,
            last_canvas_h: 800.0,
            view_mode: MapViewMode::Map,
            active_tool: MapTool::Pan,
            object_brush_mode: ObjectBrushMode::Paint,
            layers_popover_open: false,
            selected_sprite_sequence: None,
            selected_entity: None,
            tile_layer_cache: canvas::Cache::new(),
            overlay_cache: canvas::Cache::new(),
            dialog_preview: None,
            conversation: None,
        }
    }
}
