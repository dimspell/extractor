use super::message::TileGridCanvasMessage;
use super::state::{CELL_SIZE, MAX_ZOOM, MIN_ZOOM, TileGridViewState};
use dispel_core::references::tile_grid::{TileDescriptor, TileGrid};

#[test]
fn pixel_to_cell_origin() {
    let v = TileGridViewState::default();
    // At pan=0, zoom=1, pixel (0,0) → cell (0,0) → flat index 0
    assert_eq!(v.pixel_to_cell(0.0, 0.0, 249), Some(0));
    // Pixel (8,0) → cell (1,0) → flat index 1
    assert_eq!(v.pixel_to_cell(CELL_SIZE, 0.0, 249), Some(1));
    // Pixel (0,8) → cell (0,1) → flat index grid_size
    assert_eq!(v.pixel_to_cell(0.0, CELL_SIZE, 249), Some(249));
    // Pixel (8,8) → cell (1,1) → flat index 250
    assert_eq!(v.pixel_to_cell(CELL_SIZE, CELL_SIZE, 249), Some(250));
}

#[test]
fn pixel_to_cell_out_of_bounds() {
    let v = TileGridViewState::default();
    // Negative coords → None
    assert_eq!(v.pixel_to_cell(-1.0, 0.0, 249), None);
    assert_eq!(v.pixel_to_cell(0.0, -1.0, 249), None);
    // Way past the grid → None
    assert_eq!(v.pixel_to_cell(10000.0, 10000.0, 249), None);
}

#[test]
fn pixel_to_cell_with_zoom() {
    let v = TileGridViewState {
        zoom: 2.0,
        ..Default::default()
    };
    // At zoom=2, cell size is 16px. Pixel (16,0) → cell (1,0)
    assert_eq!(v.pixel_to_cell(CELL_SIZE * 2.0, 0.0, 249), Some(1));
    // Pixel (8,0) at zoom=2 → still cell (0,0)
    assert_eq!(v.pixel_to_cell(CELL_SIZE, 0.0, 249), Some(0));
}

#[test]
fn pixel_to_cell_with_pan() {
    let v = TileGridViewState {
        pan_x: -CELL_SIZE, // shifted left by 1 cell
        ..Default::default()
    };
    // Pixel (0,0) with pan=-8 → world_x = (0 - (-8)) / 8 = 1 → cell (1,0)
    assert_eq!(v.pixel_to_cell(0.0, 0.0, 249), Some(1));
}

#[test]
fn cell_to_pixel_origin() {
    let v = TileGridViewState::default();
    let (x, y) = v.cell_to_pixel(0, 249);
    assert_eq!(x, 0.0);
    assert_eq!(y, 0.0);
}

#[test]
fn cell_to_pixel_with_zoom() {
    let v = TileGridViewState {
        zoom: 2.0,
        ..Default::default()
    };
    let (x, y) = v.cell_to_pixel(1, 249);
    assert_eq!(x, CELL_SIZE * 2.0);
    assert_eq!(y, 0.0);
}

#[test]
fn cell_to_pixel_row() {
    let v = TileGridViewState::default();
    // Cell at row 1, col 0 in a 249-wide grid → flat index 249
    let (x, y) = v.cell_to_pixel(249, 249);
    assert_eq!(x, 0.0);
    assert_eq!(y, CELL_SIZE);
}

#[test]
fn clamp_zoom_bounds() {
    let mut v = TileGridViewState {
        zoom: 0.0,
        ..Default::default()
    };
    v.clamp_zoom();
    assert_eq!(v.zoom, MIN_ZOOM);

    v.zoom = 100.0;
    v.clamp_zoom();
    assert_eq!(v.zoom, MAX_ZOOM);

    v.zoom = 2.0;
    v.clamp_zoom();
    assert_eq!(v.zoom, 2.0);
}

#[test]
fn pan_message_updates_state() {
    let mut v = TileGridViewState::default();
    let mut g = TileGrid::default();
    let status = super::update::handle(TileGridCanvasMessage::Pan(10.0, 20.0), &mut v, &mut g);
    assert_eq!(v.pan_x, 10.0);
    assert_eq!(v.pan_y, 20.0);
    assert!(status.is_none());
}

#[test]
fn zoom_message_clamps() {
    let mut v = TileGridViewState::default();
    let mut g = TileGrid::default();
    // Zoom in 10× (cursor at 0,0)
    super::update::handle(TileGridCanvasMessage::Zoom(10.0, 0.0, 0.0), &mut v, &mut g);
    assert_eq!(v.zoom, 10.0_f32.min(MAX_ZOOM));
    // Zoom out 100× — should clamp to MIN_ZOOM
    super::update::handle(TileGridCanvasMessage::Zoom(0.01, 0.0, 0.0), &mut v, &mut g);
    assert_eq!(v.zoom, MIN_ZOOM);
}

#[test]
fn zoom_to_cursor_keeps_world_point_stationary() {
    // Before zoom, the cell under the cursor (100, 100) should still be
    // under the cursor after zooming. The world cell at that pixel is:
    //   world_x = (cx - pan_x) / (CELL_SIZE * old_zoom)
    // After zoom, pan must adjust so:
    //   world_x = (cx - pan_x_new) / (CELL_SIZE * new_zoom)
    // remains the same.
    let mut v = TileGridViewState {
        pan_x: 0.0,
        pan_y: 0.0,
        zoom: 1.0,
        ..Default::default()
    };
    let mut g = TileGrid {
        grid_size: 249,
        ..Default::default()
    };
    // Cursor at (100, 100), zoom in 2×
    // old_zoom = 1.0, new_zoom = 2.0
    // world_x = (100 - 0) / (8 * 1) = 12.5
    // pan_x_new = 100 - 12.5 * 16 = 100 - 200 = -100
    super::update::handle(
        TileGridCanvasMessage::Zoom(2.0, 100.0, 100.0),
        &mut v,
        &mut g,
    );
    assert_eq!(v.zoom, 2.0);
    assert!(
        (v.pan_x - (-100.0)).abs() < 0.01,
        "pan_x should be -100, got {}",
        v.pan_x
    );
    assert!(
        (v.pan_y - (-100.0)).abs() < 0.01,
        "pan_y should be -100, got {}",
        v.pan_y
    );
}

#[test]
fn zoom_to_cursor_with_existing_pan() {
    // With an initial pan, zooming should still keep the world point
    // under the cursor stationary.
    let mut v = TileGridViewState {
        pan_x: -50.0,
        pan_y: -50.0,
        zoom: 1.0,
        ..Default::default()
    };
    let mut g = TileGrid {
        grid_size: 249,
        ..Default::default()
    };
    // Cursor at (150, 150), zoom in 2×
    // world_x = (150 - (-50)) / 8 = 200/8 = 25.0
    // pan_x_new = 150 - 25.0 * 16 = 150 - 400 = -250
    super::update::handle(
        TileGridCanvasMessage::Zoom(2.0, 150.0, 150.0),
        &mut v,
        &mut g,
    );
    assert_eq!(v.zoom, 2.0);
    assert!(
        (v.pan_x - (-250.0)).abs() < 0.01,
        "pan_x should be -250, got {}",
        v.pan_x
    );
    assert!(
        (v.pan_y - (-250.0)).abs() < 0.01,
        "pan_y should be -250, got {}",
        v.pan_y
    );
}

#[test]
fn click_message_selects_cell() {
    let mut v = TileGridViewState::default();
    let mut g = TileGrid {
        grid_size: 249,
        ..Default::default()
    };
    // Click at pixel (16, 16) → cell (2, 2) → flat index 2*249+2 = 500
    let status = super::update::handle(TileGridCanvasMessage::Click(16.0, 16.0), &mut v, &mut g);
    assert_eq!(v.selected_cell, Some(500));
    assert!(status.unwrap().contains("Selected cell"));
}

#[test]
fn click_outside_grid_clears_selection() {
    let mut v = TileGridViewState {
        selected_cell: Some(42),
        ..Default::default()
    };
    let mut g = TileGrid {
        grid_size: 10,
        ..Default::default()
    };
    let status = super::update::handle(
        TileGridCanvasMessage::Click(10000.0, 10000.0),
        &mut v,
        &mut g,
    );
    assert_eq!(v.selected_cell, None);
    assert!(status.is_none());
}

#[test]
fn hover_message_sets_hovered_cell() {
    let mut v = TileGridViewState::default();
    let mut g = TileGrid {
        grid_size: 249,
        ..Default::default()
    };
    super::update::handle(TileGridCanvasMessage::Hover(0.0, 0.0), &mut v, &mut g);
    assert_eq!(v.hovered_cell, Some(0));
}

#[test]
fn edit_selected_writes_to_grid() {
    let mut v = TileGridViewState {
        selected_cell: Some(5),
        ..Default::default()
    };
    let mut g = TileGrid {
        grid_size: 10,
        data: vec![0; 100],
        ..Default::default()
    };
    let new_desc = TileDescriptor {
        object_id: 42,
        tile_index: 1337,
        render_flags: 2,
        reserved: 17,
    };
    let status = super::update::handle(
        TileGridCanvasMessage::EditSelected(new_desc),
        &mut v,
        &mut g,
    );
    let expected = super::update::encode_descriptor(new_desc);
    assert_eq!(g.data[5], expected);
    assert!(status.unwrap().contains("Edited cell 5"));
}

#[test]
fn edit_selected_without_selection_returns_error() {
    let mut v = TileGridViewState::default();
    let mut g = TileGrid::default();
    let status = super::update::handle(
        TileGridCanvasMessage::EditSelected(TileDescriptor {
            object_id: 1,
            tile_index: 0,
            render_flags: 0,
            reserved: 0,
        }),
        &mut v,
        &mut g,
    );
    assert!(status.unwrap().contains("No cell selected"));
}

#[test]
fn encode_descriptor_roundtrip() {
    // Encode then decode should recover the same fields
    let d = TileDescriptor {
        object_id: 0x03ff,  // max 10-bit
        tile_index: 0x7fff, // max 15-bit
        render_flags: 0x03, // max 2-bit
        reserved: 0x1f,     // max 5-bit
    };
    let word = super::update::encode_descriptor(d);
    let decoded = TileDescriptor::from_word(word);
    assert_eq!(decoded.object_id, d.object_id);
    assert_eq!(decoded.tile_index, d.tile_index);
    assert_eq!(decoded.render_flags, d.render_flags);
    assert_eq!(decoded.reserved, d.reserved);
}

#[test]
fn encode_descriptor_zero() {
    let d = TileDescriptor {
        object_id: 0,
        tile_index: 0,
        render_flags: 0,
        reserved: 0,
    };
    assert_eq!(super::update::encode_descriptor(d), 0);
}

#[test]
fn encode_descriptor_packing() {
    // object_id in bits 0-9, tile_index in bits 10-24, render_flags in bits 25-26, reserved in bits 27-31
    let d = TileDescriptor {
        object_id: 1,
        tile_index: 0,
        render_flags: 0,
        reserved: 0,
    };
    assert_eq!(super::update::encode_descriptor(d), 1);

    let d = TileDescriptor {
        object_id: 0,
        tile_index: 1,
        render_flags: 0,
        reserved: 0,
    };
    assert_eq!(super::update::encode_descriptor(d), 1 << 10);

    let d = TileDescriptor {
        object_id: 0,
        tile_index: 0,
        render_flags: 1,
        reserved: 0,
    };
    assert_eq!(super::update::encode_descriptor(d), 1 << 25);

    let d = TileDescriptor {
        object_id: 0,
        tile_index: 0,
        render_flags: 0,
        reserved: 1,
    };
    assert_eq!(super::update::encode_descriptor(d), 1 << 27);
}

#[test]
fn fit_to_window_computes_correct_zoom() {
    let mut v = TileGridViewState {
        last_canvas_w: 800.0,
        last_canvas_h: 600.0,
        ..Default::default()
    };
    let mut g = TileGrid {
        grid_size: 249,
        ..Default::default()
    };
    super::update::handle(TileGridCanvasMessage::FitToWindow, &mut v, &mut g);
    // Grid is 249 * 8 = 1992px. Canvas is 800×600.
    // With CANVAS_WIDTH_RATIO=0.7: avail_w = 800*0.7 = 560
    // zoom_x = 560/1992 ≈ 0.281, zoom_y = 600/1992 ≈ 0.301
    // min = 0.281
    assert!((v.zoom - 0.281).abs() < 0.01);
    // pan should center: (600 - 1992*0.281) / 2 ≈ 20
    assert!(v.pan_y.abs() < 25.0);
    assert!(!v.needs_fit);
}

#[test]
fn fit_to_window_respects_canvas_width_ratio() {
    // When the window is 1000×1000 but the canvas pane only takes
    // CANVAS_WIDTH_RATIO (0.7) of the width, the fit zoom should be
    // computed against 700px, not 1000px.
    let mut v = TileGridViewState {
        last_canvas_w: 1000.0,
        last_canvas_h: 1000.0,
        ..Default::default()
    };
    let mut g = TileGrid {
        grid_size: 249,
        ..Default::default()
    };
    super::update::handle(TileGridCanvasMessage::FitToWindow, &mut v, &mut g);
    // Grid is 249 * 8 = 1992px. Canvas pane: 1000*0.7 = 700px wide.
    // zoom_x = 700/1992 ≈ 0.351, zoom_y = 1000/1992 ≈ 0.502
    // min = 0.351
    assert!((v.zoom - 0.351).abs() < 0.01);
    // pan_x should center against avail_w (700), not full window (1000)
    let avail_w = 1000.0 * 0.7;
    let expected_pan_x = (avail_w - 1992.0 * v.zoom) * 0.5;
    assert!((v.pan_x - expected_pan_x).abs() < 0.01);
}

#[test]
fn clear_selection_works() {
    let mut v = TileGridViewState {
        selected_cell: Some(10),
        ..Default::default()
    };
    let mut g = TileGrid::default();
    super::update::handle(TileGridCanvasMessage::ClearSelection, &mut v, &mut g);
    assert_eq!(v.selected_cell, None);
}

#[test]
fn resize_updates_dimensions() {
    let mut v = TileGridViewState::default();
    let mut g = TileGrid::default();
    super::update::handle(TileGridCanvasMessage::Resize(1024.0, 768.0), &mut v, &mut g);
    assert_eq!(v.last_canvas_w, 1024.0);
    assert_eq!(v.last_canvas_h, 768.0);
}

#[test]
fn default_state_is_safe() {
    let v = TileGridViewState::default();
    assert_eq!(v.zoom, 1.0);
    assert_eq!(v.pan_x, 0.0);
    assert_eq!(v.pan_y, 0.0);
    assert!(v.selected_cell.is_none());
    assert!(v.hovered_cell.is_none());
    assert!(v.needs_fit);
}
