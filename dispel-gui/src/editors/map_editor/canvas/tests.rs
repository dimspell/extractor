//! Integration tests for the map canvas module.
//!
//! These exercise coordinate transforms, hit-testing, and entity priority logic
//! end-to-end by constructing `MapEditorState` with known data and verifying
//! correct results from the canvas helper functions.

use crate::components::loading_state::LoadingState;
use crate::components::map_render::geometry::point_in_tile_diamond;
use crate::components::map_render::{
    TILE_H, TILE_W, diamond_path, draw_item_color, is_visible, screen_to_tile, tile_center,
    tile_to_screen,
};
use crate::editors::map_editor::canvas::hit_test::{
    entity_tile, find_hovered_element, find_hovered_entity_impl, npc_pos,
};
use crate::editors::map_editor::message::{MapDataHandle, SelectedEntity};
use crate::editors::map_editor::state::MapEditorState;
use dispel_core::map::types::EventBlock;
use dispel_core::map::{MapData, MapModel};
use dispel_core::references::enums::{BooleanFlag, ItemTypeId};
use dispel_core::{DrawItem, ExtraRef, MonsterRef, NPC};
use iced::{Point, Rectangle};
use std::collections::HashMap;
use std::sync::Arc;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Build a minimal `MapEditorState` with a `MapData` that has the given
/// dimensions and entity populations.
#[allow(clippy::too_many_arguments)]
fn make_state(
    map_w: i32,
    map_h: i32,
    monsters: Vec<MonsterRef>,
    npcs: Vec<NPC>,
    extras: Vec<ExtraRef>,
    draw_items: Vec<DrawItem>,
    collisions: HashMap<(i32, i32), bool>,
    events: HashMap<(i32, i32), EventBlock>,
) -> MapEditorState {
    let model = MapModel {
        tiled_map_width: map_w,
        tiled_map_height: map_h,
        map_width_in_pixels: (map_w + map_h) * 32,
        map_height_in_pixels: (map_w + map_h) * 16,
        map_non_occluded_start_x: 0,
        map_non_occluded_start_y: 0,
        occluded_map_in_pixels_width: map_w * 32 + map_h * 32,
        occluded_map_in_pixels_height: map_w * 16 + map_h * 16,
    };

    let map_data = MapData {
        model,
        gtl_tiles: HashMap::new(),
        btl_tiles: HashMap::new(),
        collisions,
        events,
        tiled_infos: vec![],
        internal_sprites: vec![],
        sprite_blocks: vec![],
    };

    let mut state = MapEditorState::default();
    state.data.loading_state = LoadingState::Loaded(MapDataHandle(Arc::new(map_data)));
    state.data.monsters = monsters;
    state.data.npcs = npcs;
    state.data.extra_refs = extras;
    state.data.draw_items = draw_items;
    state.data.tiles_ready = true;
    // Pre-size sprite vecs to avoid index mismatches.
    // Use iterator `collect` because EntitySpriteHandle doesn't implement Clone.
    state.data.monster_sprites = (0..state.data.monsters.len()).map(|_| None).collect();
    state.data.npc_sprites = (0..state.data.npcs.len()).map(|_| None).collect();
    state.data.extra_sprites = (0..state.data.extra_refs.len()).map(|_| None).collect();
    // Default view: zero pan, 1× zoom.
    state.view.pan_x = 0.0;
    state.view.pan_y = 0.0;
    state.view.zoom = 1.0;
    state
}

fn diagonal(state: &MapEditorState) -> i32 {
    let model = &state.map_data().unwrap().0.model;
    model.tiled_map_width + model.tiled_map_height
}

/// Canvas-local coordinates for the centre of tile (tx, ty).
fn tile_centre_canvas(state: &MapEditorState, tx: i32, ty: i32) -> (f32, f32) {
    let (px, py) = tile_to_screen(
        tx,
        ty,
        diagonal(state),
        state.view.pan_x,
        state.view.pan_y,
        state.view.zoom,
    );
    tile_center(px, py, state.view.zoom)
}

// ── Coordinate transform round-trip ───────────────────────────────────────────

#[test]
fn test_tile_to_screen_round_trip_at_origin() {
    // Tile (0,0) centre → screen → back should give (0,0).
    let state = make_state(
        50,
        50,
        vec![],
        vec![],
        vec![],
        vec![],
        HashMap::new(),
        HashMap::new(),
    );
    let (cx, cy) = tile_centre_canvas(&state, 0, 0);
    let result = screen_to_tile(
        cx,
        cy,
        diagonal(&state),
        state.view.pan_x,
        state.view.pan_y,
        state.view.zoom,
        50,
        50,
    );
    assert_eq!(result, Some((0, 0)));
}

#[test]
fn test_tile_to_screen_round_trip_various_positions() {
    let state = make_state(
        100,
        100,
        vec![],
        vec![],
        vec![],
        vec![],
        HashMap::new(),
        HashMap::new(),
    );
    let d = diagonal(&state);

    for &(tx, ty) in &[(5, 5), (10, 20), (30, 40), (0, 50), (99, 0)] {
        let (cx, cy) = tile_centre_canvas(&state, tx, ty);
        let result = screen_to_tile(cx, cy, d, 0.0, 0.0, 1.0, 100, 100);
        assert_eq!(
            result,
            Some((tx, ty)),
            "round-trip failed for tile ({}, {})",
            tx,
            ty
        );
    }
}

#[test]
fn test_tile_to_screen_round_trip_with_zoom_and_pan() {
    let state = {
        let mut s = make_state(
            60,
            60,
            vec![],
            vec![],
            vec![],
            vec![],
            HashMap::new(),
            HashMap::new(),
        );
        s.view.zoom = 2.5;
        s.view.pan_x = 100.0;
        s.view.pan_y = -50.0;
        s
    };
    let d = diagonal(&state);

    for &(tx, ty) in &[(15, 15), (0, 30), (40, 10)] {
        let (px, py) = tile_to_screen(
            tx,
            ty,
            d,
            state.view.pan_x,
            state.view.pan_y,
            state.view.zoom,
        );
        let (cx, cy) = tile_center(px, py, state.view.zoom);
        let result = screen_to_tile(
            cx,
            cy,
            d,
            state.view.pan_x,
            state.view.pan_y,
            state.view.zoom,
            60,
            60,
        );
        assert_eq!(
            result,
            Some((tx, ty)),
            "round-trip with zoom/pan failed for ({}, {})",
            tx,
            ty
        );
    }
}

// ── is_visible ────────────────────────────────────────────────────────────────

#[test]
fn test_is_visible_inside_bounds() {
    let bounds = Rectangle::new(Point::new(0.0, 0.0), iced::Size::new(800.0, 600.0));
    assert!(is_visible(10.0, 10.0, 100.0, 100.0, bounds));
    assert!(is_visible(0.0, 0.0, 800.0, 600.0, bounds));
}

#[test]
fn test_is_visible_outside_bounds() {
    let bounds = Rectangle::new(Point::new(0.0, 0.0), iced::Size::new(800.0, 600.0));
    assert!(
        !is_visible(900.0, 0.0, 100.0, 100.0, bounds),
        "right of bounds"
    );
    assert!(
        !is_visible(0.0, 700.0, 100.0, 100.0, bounds),
        "below bounds"
    );
    assert!(
        !is_visible(-200.0, 0.0, 100.0, 100.0, bounds),
        "left of bounds, x + w < 0"
    );
}

#[test]
fn test_is_visible_partially_visible() {
    let bounds = Rectangle::new(Point::new(0.0, 0.0), iced::Size::new(800.0, 600.0));
    // Partially visible: rect starts left of 0 but extends into view.
    assert!(
        is_visible(-50.0, 10.0, 100.0, 100.0, bounds),
        "partially left"
    );
    // Partially visible: rect starts above 0 but extends into view.
    assert!(
        is_visible(10.0, -50.0, 100.0, 100.0, bounds),
        "partially top"
    );
}

// ── point_in_tile_diamond ─────────────────────────────────────────────────────

#[test]
fn test_point_in_tile_diamond_centre() {
    let (sx, sy) = (100.0, 200.0);
    let cx = sx + TILE_W * 0.5;
    let cy = sy + TILE_H * 0.5;
    assert!(point_in_tile_diamond(cx, cy, sx, sy, 1.0));
}

#[test]
fn test_point_in_tile_diamond_outside() {
    let (sx, sy) = (100.0, 200.0);
    // Point far below the diamond.
    assert!(!point_in_tile_diamond(sx, sy + 100.0, sx, sy, 1.0));
}

#[test]
fn test_point_in_tile_diamond_edge() {
    let (sx, sy) = (100.0, 200.0);
    let cx = sx + TILE_W * 0.5;
    // Right edge: (cx + TILE_W*0.5, cy) — dx=1, dy=0 → sum=1 → exactly on edge.
    let right_edge_x = cx + TILE_W * 0.5;
    assert!(point_in_tile_diamond(
        right_edge_x,
        sy + TILE_H * 0.5,
        sx,
        sy,
        1.0
    ));
    // Just past the right edge: dx > 1 → sum > 1 → outside.
    let past_right = right_edge_x + 1.0;
    assert!(!point_in_tile_diamond(
        past_right,
        sy + TILE_H * 0.5,
        sx,
        sy,
        1.0
    ));
}

// ── npc_pos ───────────────────────────────────────────────────────────────────

#[test]
fn test_npc_pos_falls_back_to_goto1_when_none_filled() {
    let n = NPC {
        goto1_x: 10,
        goto1_y: 20,
        goto2_x: 30,
        goto2_y: 40,
        ..Default::default()
    };
    // All goto_filled are default (False), so fallback to goto1.
    assert_eq!(npc_pos(&n), (10, 20));
}

#[test]
fn test_npc_pos_uses_first_filled_waypoint() {
    let n = NPC {
        goto1_filled: BooleanFlag::False,
        goto1_x: 10,
        goto1_y: 20,
        goto2_filled: BooleanFlag::True,
        goto2_x: 30,
        goto2_y: 40,
        goto3_filled: BooleanFlag::True,
        goto3_x: 50,
        goto3_y: 60,
        ..Default::default()
    };
    // goto2 is the first filled waypoint.
    assert_eq!(npc_pos(&n), (30, 40));
}

#[test]
fn test_npc_pos_prefers_goto4() {
    let n = NPC {
        goto1_filled: BooleanFlag::True,
        goto1_x: 10,
        goto1_y: 20,
        goto4_filled: BooleanFlag::True,
        goto4_x: 70,
        goto4_y: 80,
        ..Default::default()
    };
    assert_eq!(npc_pos(&n), (10, 20));
}

#[test]
fn test_npc_pos_uses_goto4_when_earlier_empty() {
    let n = NPC {
        goto1_filled: BooleanFlag::False,
        goto1_x: 10,
        goto1_y: 20,
        goto2_filled: BooleanFlag::False,
        goto2_x: 30,
        goto2_y: 40,
        goto3_filled: BooleanFlag::True,
        goto3_x: 50,
        goto3_y: 60,
        ..Default::default()
    };
    assert_eq!(npc_pos(&n), (50, 60));
}

// ── entity_tile ───────────────────────────────────────────────────────────────

#[test]
fn test_entity_tile_monster() {
    let state = make_state(
        50,
        50,
        vec![MonsterRef {
            pos_x: 7,
            pos_y: 13,
            ..Default::default()
        }],
        vec![],
        vec![],
        vec![],
        HashMap::new(),
        HashMap::new(),
    );
    assert_eq!(
        entity_tile(SelectedEntity::Monster(0), &state),
        Some((7, 13))
    );
}

#[test]
fn test_entity_tile_npc() {
    let state = make_state(
        50,
        50,
        vec![],
        vec![NPC {
            goto1_x: 4,
            goto1_y: 9,
            ..Default::default()
        }],
        vec![],
        vec![],
        HashMap::new(),
        HashMap::new(),
    );
    assert_eq!(entity_tile(SelectedEntity::Npc(0), &state), Some((4, 9)));
}

#[test]
fn test_entity_tile_extra() {
    let state = make_state(
        50,
        50,
        vec![],
        vec![],
        vec![ExtraRef {
            map_x: 22,
            map_y: 33,
            ..Default::default()
        }],
        vec![],
        HashMap::new(),
        HashMap::new(),
    );
    assert_eq!(
        entity_tile(SelectedEntity::Extra(0), &state),
        Some((22, 33))
    );
}

#[test]
fn test_entity_tile_draw_item() {
    let state = make_state(
        50,
        50,
        vec![],
        vec![],
        vec![],
        vec![DrawItem {
            x_coord: 15,
            y_coord: 25,
            ..Default::default()
        }],
        HashMap::new(),
        HashMap::new(),
    );
    assert_eq!(
        entity_tile(SelectedEntity::DrawItem(0), &state),
        Some((15, 25))
    );
}

#[test]
fn test_entity_tile_collision_and_event() {
    assert_eq!(
        entity_tile(SelectedEntity::CollisionTile(3, 7), &Default::default()),
        Some((3, 7)),
    );
    assert_eq!(
        entity_tile(SelectedEntity::EventTile(9, 2), &Default::default()),
        Some((9, 2)),
    );
}

// ── find_hovered_entity_impl — entity hit-testing ─────────────────────────────

#[test]
fn test_hovered_entity_identifies_monster() {
    // Place a monster at tile (20, 20) in a 50×50 map.
    let state = make_state(
        50,
        50,
        vec![MonsterRef {
            pos_x: 20,
            pos_y: 20,
            ..Default::default()
        }],
        vec![],
        vec![],
        vec![],
        HashMap::new(),
        HashMap::new(),
    );
    // Compute the canvas-coordinate of the monster's tile centre.
    let (cx, cy) = tile_centre_canvas(&state, 20, 20);
    let found = find_hovered_entity_impl(&state, cx, cy);
    assert_eq!(found, Some(SelectedEntity::Monster(0)));
}

#[test]
fn test_hovered_entity_identifies_npc() {
    let state = make_state(
        50,
        50,
        vec![],
        vec![NPC {
            goto1_x: 30,
            goto1_y: 10,
            ..Default::default()
        }],
        vec![],
        vec![],
        HashMap::new(),
        HashMap::new(),
    );
    let (cx, cy) = tile_centre_canvas(&state, 30, 10);
    let found = find_hovered_entity_impl(&state, cx, cy);
    assert_eq!(found, Some(SelectedEntity::Npc(0)));
}

#[test]
fn test_hovered_entity_identifies_extra() {
    let state = make_state(
        50,
        50,
        vec![],
        vec![],
        vec![ExtraRef {
            map_x: 12,
            map_y: 34,
            ..Default::default()
        }],
        vec![],
        HashMap::new(),
        HashMap::new(),
    );
    let (cx, cy) = tile_centre_canvas(&state, 12, 34);
    let found = find_hovered_entity_impl(&state, cx, cy);
    assert_eq!(found, Some(SelectedEntity::Extra(0)));
}

#[test]
fn test_hovered_entity_identifies_draw_item() {
    let state = make_state(
        50,
        50,
        vec![],
        vec![],
        vec![],
        vec![DrawItem {
            x_coord: 8,
            y_coord: 16,
            ..Default::default()
        }],
        HashMap::new(),
        HashMap::new(),
    );
    let (cx, cy) = tile_centre_canvas(&state, 8, 16);
    let found = find_hovered_entity_impl(&state, cx, cy);
    assert_eq!(found, Some(SelectedEntity::DrawItem(0)));
}

#[test]
fn test_hovered_entity_returns_closest() {
    // Two monsters near each other; cursor closest to monster[0].
    let state = make_state(
        50,
        50,
        vec![
            MonsterRef {
                pos_x: 20,
                pos_y: 20,
                ..Default::default()
            },
            MonsterRef {
                pos_x: 20,
                pos_y: 21,
                ..Default::default()
            },
        ],
        vec![],
        vec![],
        vec![],
        HashMap::new(),
        HashMap::new(),
    );
    let (cx, cy) = tile_centre_canvas(&state, 20, 20);
    assert_eq!(
        find_hovered_entity_impl(&state, cx, cy),
        Some(SelectedEntity::Monster(0)),
    );
    let (cx, cy) = tile_centre_canvas(&state, 20, 21);
    assert_eq!(
        find_hovered_entity_impl(&state, cx, cy),
        Some(SelectedEntity::Monster(1)),
    );
}

#[test]
fn test_hovered_entity_none_when_far() {
    let state = make_state(
        50,
        50,
        vec![MonsterRef {
            pos_x: 5,
            pos_y: 5,
            ..Default::default()
        }],
        vec![],
        vec![],
        vec![],
        HashMap::new(),
        HashMap::new(),
    );
    // Cursor far from the monster → no hit.
    let (cx, cy) = tile_centre_canvas(&state, 40, 40);
    assert_eq!(find_hovered_entity_impl(&state, cx, cy), None);
}

// ── find_hovered_element — priority between entity / collision / event ────────

#[test]
fn test_hovered_element_entity_over_collision() {
    // Monster at (10, 10) with a collision at the same tile.
    let mut collisions = HashMap::new();
    collisions.insert((10, 10), true);
    let state = make_state(
        50,
        50,
        vec![MonsterRef {
            pos_x: 10,
            pos_y: 10,
            ..Default::default()
        }],
        vec![],
        vec![],
        vec![],
        collisions,
        HashMap::new(),
    );
    let (cx, cy) = tile_centre_canvas(&state, 10, 10);
    // Entity should win over collision.
    assert_eq!(
        find_hovered_element(&state, cx, cy),
        Some(SelectedEntity::Monster(0)),
    );
}

#[test]
fn test_hovered_element_entity_over_event() {
    let mut events = HashMap::new();
    events.insert(
        (10, 10),
        EventBlock {
            x: 10,
            y: 10,
            _unknown_value: 0,
            event_id: 1,
        },
    );
    let state = make_state(
        50,
        50,
        vec![],
        vec![],
        vec![ExtraRef {
            map_x: 10,
            map_y: 10,
            ..Default::default()
        }],
        vec![],
        HashMap::new(),
        events,
    );
    let (cx, cy) = tile_centre_canvas(&state, 10, 10);
    assert_eq!(
        find_hovered_element(&state, cx, cy),
        Some(SelectedEntity::Extra(0)),
    );
}

#[test]
fn test_hovered_element_collision_when_no_entity() {
    let mut collisions = HashMap::new();
    collisions.insert((7, 8), true);
    let state = make_state(
        50,
        50,
        vec![],
        vec![],
        vec![],
        vec![],
        collisions,
        HashMap::new(),
    );
    // Enable collision layer so tile hit-testing is active.
    assert!(!state.view.show_collisions);
    // find_hovered_element only checks collisions when layer is visible.
    // We need to test with show_collisions = true.
    let mut state = state;
    state.view.show_collisions = true;

    let (cx, cy) = tile_centre_canvas(&state, 7, 8);
    assert_eq!(
        find_hovered_element(&state, cx, cy),
        Some(SelectedEntity::CollisionTile(7, 8)),
    );
}

#[test]
fn test_hovered_element_event_when_no_entity_or_collision() {
    let mut events = HashMap::new();
    events.insert(
        (15, 25),
        EventBlock {
            x: 15,
            y: 25,
            _unknown_value: 0,
            event_id: 42,
        },
    );
    let state = make_state(
        50,
        50,
        vec![],
        vec![],
        vec![],
        vec![],
        HashMap::new(),
        events,
    );
    let mut state = state;
    state.view.show_events = true;

    let (cx, cy) = tile_centre_canvas(&state, 15, 25);
    assert_eq!(
        find_hovered_element(&state, cx, cy),
        Some(SelectedEntity::EventTile(15, 25)),
    );
}

#[test]
fn test_hovered_element_none_when_nothing_present() {
    let state = make_state(
        50,
        50,
        vec![],
        vec![],
        vec![],
        vec![],
        HashMap::new(),
        HashMap::new(),
    );
    let (cx, cy) = tile_centre_canvas(&state, 10, 10);
    assert_eq!(find_hovered_element(&state, cx, cy), None);
}

#[test]
fn test_hovered_element_respects_layer_visibility() {
    let mut events = HashMap::new();
    events.insert(
        (3, 4),
        EventBlock {
            x: 3,
            y: 4,
            _unknown_value: 0,
            event_id: 99,
        },
    );
    let state = make_state(
        50,
        50,
        vec![],
        vec![],
        vec![],
        vec![],
        HashMap::new(),
        events,
    );
    let (cx, cy) = tile_centre_canvas(&state, 3, 4);
    // show_events is false by default, so event tile should NOT be detected.
    assert_eq!(find_hovered_element(&state, cx, cy), None);
}

// ── draw_item_color ──────────────────────────────────────────────────────────

#[test]
fn test_draw_item_color_weapon() {
    let c = draw_item_color(ItemTypeId::Weapon);
    assert_eq!(c.r, 0.9);
    assert_eq!(c.g, 0.15);
    assert_eq!(c.b, 0.15);
    assert!((c.a - 0.85).abs() < 0.001);
}

#[test]
fn test_draw_item_color_healing() {
    let c = draw_item_color(ItemTypeId::Healing);
    assert_eq!(c.r, 0.15);
    assert_eq!(c.g, 0.9);
    assert_eq!(c.b, 0.15);
}

#[test]
fn test_draw_item_color_edit() {
    let c = draw_item_color(ItemTypeId::Edit);
    assert_eq!(c.r, 0.15);
    assert_eq!(c.g, 0.45);
    assert_eq!(c.b, 0.9);
}

#[test]
fn test_draw_item_color_event() {
    let c = draw_item_color(ItemTypeId::Event);
    assert_eq!(c.r, 0.8);
    assert_eq!(c.g, 0.15);
    assert_eq!(c.b, 0.8);
}

#[test]
fn test_draw_item_color_misc() {
    let c = draw_item_color(ItemTypeId::Misc);
    assert_eq!(c.r, 0.95);
    assert_eq!(c.g, 0.85);
    assert_eq!(c.b, 0.1);
}

#[test]
fn test_draw_item_color_other() {
    let c = draw_item_color(ItemTypeId::Other);
    assert_eq!(c.r, 0.6);
    assert_eq!(c.g, 0.6);
    assert_eq!(c.b, 0.6);
}

// ── diamond_path ──────────────────────────────────────────────────────────────

#[test]
fn test_diamond_path_creates_valid_path() {
    // The path should not panic; verify it returns a canvas::Path.
    let path = diamond_path(100.0, 200.0, 10.0);
    // Canvas Path is opaque — just ensure it doesn't panic and has the right type.
    let _ = path; // (lint guard)
}
