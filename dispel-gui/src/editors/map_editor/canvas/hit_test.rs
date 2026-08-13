// ── Hit-testing: find what's under the cursor ────────────────────────────────

use crate::components::map_render::{HOVER_RADIUS_PX, screen_to_tile, tile_world_center};
use crate::editors::map_editor::message::SelectedEntity;
use crate::editors::map_editor::state::MapEditorState;

/// Free function so both `MapCanvas` and `MapCanvasOverlaysLayer::draw` can use it
/// without going through the message pipeline.  Also called from the update handler
/// for `CanvasClicked` so click and hover share the same hit-test logic.
#[allow(clippy::question_mark)]
pub fn find_hovered_entity_impl(
    state: &MapEditorState,
    cx: f32,
    cy: f32,
) -> Option<SelectedEntity> {
    let Some(map_handle) = state.map_data() else {
        return None;
    };
    let model = &map_handle.0.model;
    let diagonal = model.tiled_map_width + model.tiled_map_height;

    // Convert canvas-local to world pixel space.
    let world_x = (cx - state.view.pan_x) / state.view.zoom;
    let world_y = (cy - state.view.pan_y) / state.view.zoom;

    let r2 = HOVER_RADIUS_PX * HOVER_RADIUS_PX;
    let mut best: Option<(f32, SelectedEntity)> = None;

    for (i, m) in state.data.monsters.iter().enumerate() {
        let (wx, wy) = tile_world_center(m.pos_x, m.pos_y, diagonal);
        let d2 = (world_x - wx).powi(2) + (world_y - wy).powi(2);
        if d2 < r2 && best.as_ref().is_none_or(|(bd, _)| d2 < *bd) {
            best = Some((d2, SelectedEntity::Monster(i)));
        }
    }
    for (i, n) in state.data.npcs.iter().enumerate() {
        let (nx, ny) = npc_pos(n);
        let (wx, wy) = tile_world_center(nx, ny, diagonal);
        let d2 = (world_x - wx).powi(2) + (world_y - wy).powi(2);
        if d2 < r2 && best.as_ref().is_none_or(|(bd, _)| d2 < *bd) {
            best = Some((d2, SelectedEntity::Npc(i)));
        }
    }
    for (i, e) in state.data.extra_refs.iter().enumerate() {
        let (wx, wy) = tile_world_center(e.map_x, e.map_y, diagonal);
        let d2 = (world_x - wx).powi(2) + (world_y - wy).powi(2);
        if d2 < r2 && best.as_ref().is_none_or(|(bd, _)| d2 < *bd) {
            best = Some((d2, SelectedEntity::Extra(i)));
        }
    }
    for (i, d) in state.data.draw_items.iter().enumerate() {
        let (wx, wy) = tile_world_center(d.x_coord, d.y_coord, diagonal);
        let d2 = (world_x - wx).powi(2) + (world_y - wy).powi(2);
        if d2 < r2 && best.as_ref().is_none_or(|(bd, _)| d2 < *bd) {
            best = Some((d2, SelectedEntity::DrawItem(i)));
        }
    }

    best.map(|(_, e)| e)
}

/// Find the collision tile under the cursor (if any).  Returns `(tx, ty)`.
fn find_hovered_collision_tile(state: &MapEditorState, cx: f32, cy: f32) -> Option<(i32, i32)> {
    let map_handle = state.map_data()?;
    let map_data = &map_handle.0;
    let model = &map_data.model;
    let diagonal = model.tiled_map_width + model.tiled_map_height;

    let (tile_x, tile_y) = screen_to_tile(
        cx,
        cy,
        diagonal,
        state.view.pan_x,
        state.view.pan_y,
        state.view.zoom,
        model.tiled_map_width,
        model.tiled_map_height,
    )?;

    // When the collision layer is visible, every tile is paintable.
    // Return the tile even if it doesn't have a collision yet — the
    // click handler will toggle it (false → true).
    Some((tile_x, tile_y))
}

/// Find the event tile under the cursor (if any).  Returns `(tx, ty)`.
fn find_hovered_event_tile(state: &MapEditorState, cx: f32, cy: f32) -> Option<(i32, i32)> {
    let map_handle = state.map_data()?;
    let map_data = &map_handle.0;
    let model = &map_data.model;
    let diagonal = model.tiled_map_width + model.tiled_map_height;

    let (tile_x, tile_y) = screen_to_tile(
        cx,
        cy,
        diagonal,
        state.view.pan_x,
        state.view.pan_y,
        state.view.zoom,
        model.tiled_map_width,
        model.tiled_map_height,
    )?;

    // When the event layer is visible, every tile is a potential event target.
    // Return the tile regardless of whether it already has an event — the
    // inspector will show either the event editor or a "Create Event" button.
    Some((tile_x, tile_y))
}

/// Find what's under the cursor, with priority: entity > collision tile > event tile.
pub fn find_hovered_element(state: &MapEditorState, cx: f32, cy: f32) -> Option<SelectedEntity> {
    // 1. Try entities (existing logic)
    if let Some(entity) = find_hovered_entity_impl(state, cx, cy) {
        return Some(entity);
    }
    // 2. Try collision tiles (only when collision layer is visible)
    if state.view.show_collisions
        && let Some((tx, ty)) = find_hovered_collision_tile(state, cx, cy)
    {
        return Some(SelectedEntity::CollisionTile(tx, ty));
    }
    // 3. Try event tiles (only when event layer is visible)
    if state.view.show_events
        && let Some((tx, ty)) = find_hovered_event_tile(state, cx, cy)
    {
        return Some(SelectedEntity::EventTile(tx, ty));
    }
    None
}

/// Return the tile coordinates for an entity.
pub fn entity_tile(sel: SelectedEntity, state: &MapEditorState) -> Option<(i32, i32)> {
    match sel {
        SelectedEntity::Monster(i) => state.data.monsters.get(i).map(|m| (m.pos_x, m.pos_y)),
        SelectedEntity::Npc(i) => state.data.npcs.get(i).map(|n| {
            let (x, y) = npc_pos(n);
            (x, y)
        }),
        SelectedEntity::Extra(i) => state.data.extra_refs.get(i).map(|e| (e.map_x, e.map_y)),
        SelectedEntity::DrawItem(i) => state.data.draw_items.get(i).map(|d| (d.x_coord, d.y_coord)),
        SelectedEntity::CollisionTile(tx, ty) | SelectedEntity::EventTile(tx, ty) => Some((tx, ty)),
    }
}

/// First active waypoint (or goto1 fallback) for an NPC.
pub fn npc_pos(n: &dispel_core::NPC) -> (i32, i32) {
    [
        (n.goto1_filled, n.goto1_x, n.goto1_y),
        (n.goto2_filled, n.goto2_x, n.goto2_y),
        (n.goto3_filled, n.goto3_x, n.goto3_y),
        (n.goto4_filled, n.goto4_x, n.goto4_y),
    ]
    .iter()
    .find(|(filled, _, _)| i32::from(*filled) != 0)
    .map(|&(_, x, y)| (x, y))
    .unwrap_or((n.goto1_x, n.goto1_y))
}
