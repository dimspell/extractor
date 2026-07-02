use super::canvas::{MapCanvasOverlaysLayer, MapCanvasTilesLayer};
use super::message::{MapEditorMessage, MapLayer, MapViewMode};
use crate::app::App;
use crate::components::loading_state::LoadingState;
use crate::message::{Message, MessageExt};
use crate::style;
use gui_widgets::components::modal::modal;
use iced::widget::{button, canvas, column, container, progress_bar, row, stack, text, toggler};
use iced::{Element, Fill};

mod dialog_preview;
mod fields;
mod inspector;
mod sprites;

pub fn view(app: &App) -> Element<'_, Message> {
    let tab_id = match app.state.workspace.active() {
        Some(tab) => tab.id,
        None => return text("No active tab").into(),
    };

    let state = match app.state.editors.map_editors.get(&tab_id) {
        Some(s) => s,
        None => {
            return container(
                text("Map editor not initialised — reopen the file.")
                    .size(12)
                    .style(style::subtle_text),
            )
            .padding(24)
            .accessible_label("Map editor")
            .into()
        }
    };

    match &state.data.loading_state {
        LoadingState::Idle => container(
            text("Map file not loaded.")
                .size(12)
                .style(style::subtle_text),
        )
        .padding(24)
        .accessible_label("Map editor")
        .into(),

        LoadingState::Loading => container(
            column![
                text("Loading map…").size(12).style(style::subtle_text),
                progress_bar(0.0..=1.0, 0.5).style(style::primary_progress_bar),
            ]
            .spacing(8)
            .padding(24),
        )
        .width(Fill)
        .accessible_label("Map editor")
        .into(),

        LoadingState::Failed(err) => container(
            column![
                text("Failed to load map")
                    .size(13)
                    .color(iced::Color::from_rgb(0.8, 0.2, 0.2)),
                text(err.as_str()).size(11).style(style::subtle_text),
            ]
            .spacing(8)
            .padding(24),
        )
        .accessible_label("Map editor")
        .into(),

        LoadingState::Loaded(map_handle) => {
            let map_data = &map_handle.0;
            let model = &map_data.model;
            let gtl_count = map_data.gtl_tiles.len();
            let btl_count = map_data.btl_tiles.len();

            let path_label = state
                .data
                .map_path
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            // ── Info row ─────────────────────────────────────────────────
            let info_row = row![
                info_cell("File", &path_label),
                info_cell(
                    "Tiles (W×H)",
                    &format!("{}×{}", model.tiled_map_width, model.tiled_map_height)
                ),
                info_cell("GTL", &gtl_count.to_string()),
                info_cell("BTL", &btl_count.to_string()),
                info_cell(
                    "Handles",
                    &format!(
                        "{}/{}",
                        state.data.gtl_handles.len(),
                        state.data.btl_handles.len()
                    )
                ),
                info_cell(
                    "Entities",
                    &format!(
                        "{}M {}N {}O {}D",
                        state.data.monsters.len(),
                        state.data.npcs.len(),
                        state.data.extra_refs.len(),
                        state.data.draw_items.len()
                    )
                ),
                info_cell(
                    "Tiles",
                    &format!(
                        "{} GTL + {} BTL loaded",
                        state.data.gtl_handles.len(),
                        state.data.btl_handles.len()
                    )
                ),
            ]
            .spacing(16)
            .padding([8, 16]);

            // ── Layer toggles ─────────────────────────────────────────────
            let tile_status = if state.data.tiles_ready {
                text("").size(10).style(style::subtle_text)
            } else {
                text("Decoding tiles…").size(10).style(style::subtle_text)
            };

            let layer_row = row![
                text("Layers:").size(11).style(style::subtle_text),
                layer_toggle(
                    "Ground",
                    state.view.show_ground,
                    tab_id,
                    MapLayer::Ground,
                    None
                ),
                layer_toggle(
                    "Buildings",
                    state.view.show_buildings,
                    tab_id,
                    MapLayer::Buildings,
                    None
                ),
                layer_toggle(
                    "Roofs",
                    state.view.show_roofs,
                    tab_id,
                    MapLayer::Roofs,
                    None
                ),
                layer_toggle(
                    "Sprites",
                    state.view.show_internal_sprites,
                    tab_id,
                    MapLayer::InternalSprites,
                    None
                ),
                layer_toggle(
                    "Collisions",
                    state.view.show_collisions,
                    tab_id,
                    MapLayer::Collisions,
                    None
                ),
                layer_toggle(
                    "Events",
                    state.view.show_events,
                    tab_id,
                    MapLayer::Events,
                    None
                ),
                layer_toggle(
                    "Monsters",
                    state.view.show_monsters,
                    tab_id,
                    MapLayer::Monsters,
                    Some(state.data.monsters.len())
                ),
                layer_toggle(
                    "NPCs",
                    state.view.show_npcs,
                    tab_id,
                    MapLayer::Npcs,
                    Some(state.data.npcs.len())
                ),
                layer_toggle(
                    "Waypoints",
                    state.view.show_npc_waypoints,
                    tab_id,
                    MapLayer::NpcWaypoints,
                    None
                ),
                layer_toggle(
                    "Objects",
                    state.view.show_objects,
                    tab_id,
                    MapLayer::Objects,
                    Some(state.data.extra_refs.len())
                ),
                layer_toggle(
                    "Items",
                    state.view.show_draw_items,
                    tab_id,
                    MapLayer::DrawItems,
                    Some(state.data.draw_items.len())
                ),
            ]
            .spacing(12)
            .padding([6, 16])
            .align_y(iced::Alignment::Center);

            // ── Action buttons row ─────────────────────────────────────────
            let can_undo = !state.data.undo_stack.is_empty();
            let can_redo = !state.data.redo_stack.is_empty();
            let save_label = if state.data.is_saving {
                "Saving…"
            } else if state.data.dirty {
                "Save*"
            } else {
                "Save"
            };
            let mut save_btn = button(text(save_label).size(11)).padding([3, 8]);
            if state.data.dirty && !state.data.is_saving {
                save_btn =
                    save_btn.on_press(Message::map_editor(MapEditorMessage::SaveMap(tab_id)));
            }

            let mut undo_btn = button(text("↩ Undo").size(11)).padding([3, 8]);
            if can_undo {
                undo_btn = undo_btn.on_press(Message::map_editor(MapEditorMessage::Undo(tab_id)));
            }

            let mut redo_btn = button(text("↪ Redo").size(11)).padding([3, 8]);
            if can_redo {
                redo_btn = redo_btn.on_press(Message::map_editor(MapEditorMessage::Redo(tab_id)));
            }

            let export_label = if state.data.is_exporting {
                "Exporting…"
            } else {
                "Export PNG"
            };
            let mut export_btn = button(text(export_label).size(11)).padding([3, 8]);
            if !state.data.is_exporting {
                export_btn =
                    export_btn.on_press(Message::map_editor(MapEditorMessage::ExportImage(tab_id)));
            }

            let mut tmx_btn = button(text("Export TMX…").size(11)).padding([3, 8]);
            if !state.data.is_exporting {
                tmx_btn =
                    tmx_btn.on_press(Message::map_editor(MapEditorMessage::ExportTmx(tab_id)));
            }

            let status_text = if let Some(msg) = &state.data.status_msg {
                text(msg.as_str()).size(10).style(style::subtle_text)
            } else {
                text("").size(10).style(style::subtle_text)
            };

            let action_row = row![save_btn, undo_btn, redo_btn, export_btn, tmx_btn, status_text,]
                .spacing(6)
                .padding([4, 16])
                .align_y(iced::Alignment::Center);

            let mode_tab_row = row![
                button(text("Map").size(11))
                    .on_press(Message::map_editor(MapEditorMessage::SwitchViewMode(
                        tab_id,
                        MapViewMode::Map
                    )))
                    .padding([3, 10])
                    .style(if state.view.view_mode == MapViewMode::Map {
                        style::active_chip
                    } else {
                        style::chip
                    }),
                button(text("Sprites").size(11))
                    .on_press(Message::map_editor(MapEditorMessage::SwitchViewMode(
                        tab_id,
                        MapViewMode::Sprites
                    )))
                    .padding([3, 10])
                    .style(if state.view.view_mode == MapViewMode::Sprites {
                        style::active_chip
                    } else {
                        style::chip
                    }),
            ]
            .spacing(4)
            .padding([6, 16]);

            let toolbar = container(
                column![
                    row![mode_tab_row, action_row].spacing(0),
                    row![info_row].spacing(0),
                    row![layer_row, tile_status]
                        .spacing(16)
                        .align_y(iced::Alignment::Center),
                ]
                .spacing(0),
            )
            .width(Fill)
            .style(style::toolbar_container)
            .accessible_label("Map editor toolbar");

            // ── Canvas for tile layers, sprites (images) ───────────────────────
            let tiles_canvas = canvas(MapCanvasTilesLayer { state, tab_id })
                .width(Fill)
                .height(Fill);

            // ── Canvas for overlay elements (primitives) ───────────────────────
            let overlays_canvas = canvas(MapCanvasOverlaysLayer { state, tab_id })
                .width(Fill)
                .height(Fill);

            // Stack: overlays on top of tiles (primitives draw above images)
            let map_canvas = stack![tiles_canvas, overlays_canvas]
                .width(Fill)
                .height(Fill);

            // ── Floating zoom controls (right side, Google Maps style) ───
            let zoom_controls = container(
                column![
                    button(text("+").size(14))
                        .on_press(Message::map_editor(MapEditorMessage::ZoomChanged(
                            tab_id,
                            1.25,
                            f32::NAN,
                            f32::NAN
                        )))
                        .padding([5, 10])
                        .style(style::browse_button),
                    text(format!("{:.0}%", state.view.zoom * 100.0)).size(10),
                    button(text("−").size(14))
                        .on_press(Message::map_editor(MapEditorMessage::ZoomChanged(
                            tab_id,
                            1.0 / 1.25,
                            f32::NAN,
                            f32::NAN
                        )))
                        .padding([5, 10])
                        .style(style::browse_button),
                    button(text("⊡").size(11))
                        .on_press(Message::map_editor(MapEditorMessage::FitToWindow(tab_id)))
                        .padding([5, 10])
                        .style(style::browse_button),
                ]
                .spacing(4)
                .align_x(iced::Alignment::Center),
            )
            .padding(8)
            .width(Fill)
            .height(Fill)
            .align_x(iced::alignment::Horizontal::Right)
            .align_y(iced::alignment::Vertical::Center);

            let canvas_with_overlay = stack![map_canvas, zoom_controls].width(Fill).height(Fill);

            // ── Body: map canvas or sprite browser ───────────────────────
            let body: Element<'_, Message> = match state.view.view_mode {
                MapViewMode::Map => match state.view.selected_entity {
                    Some(sel) => {
                        let inspector =
                            inspector::build_inspector(state, tab_id, sel, &app.state.lookups);
                        row![canvas_with_overlay, inspector]
                            .width(Fill)
                            .height(Fill)
                            .into()
                    }
                    None => canvas_with_overlay.into(),
                },
                MapViewMode::Sprites => sprites::view_sprite_browser(state, tab_id),
            };

            let base: Element<'_, Message> = column![toolbar, body]
                .spacing(0)
                .width(Fill)
                .height(Fill)
                .accessible_label("Map editor")
                .into();

            // Wrap in dialog preview modal if open
            if let Some(ref preview) = state.view.dialog_preview {
                modal(
                    base,
                    dialog_preview::view_dialog_preview(state, tab_id, preview),
                    move || Message::map_editor(MapEditorMessage::HideDialogPreview(tab_id)),
                    0.5,
                )
            } else {
                base
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn info_cell<'a>(label: &'static str, value: &str) -> Element<'a, Message> {
    column![
        text(label).size(10).style(style::subtle_text),
        text(value.to_string()).size(11),
    ]
    .spacing(2)
    .into()
}

fn layer_toggle(
    label: &'static str,
    is_on: bool,
    tab_id: usize,
    layer: MapLayer,
    count: Option<usize>,
) -> Element<'static, Message> {
    let label_str: String = match count {
        Some(n) => format!("{} ({})", label, n),
        None => label.to_string(),
    };
    toggler(is_on)
        .label(label_str)
        .text_size(11)
        .size(12)
        .on_toggle(move |_| Message::map_editor(MapEditorMessage::LayerToggled(tab_id, layer)))
        .into()
}
