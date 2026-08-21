use super::canvas::{MapCanvasOverlaysLayer, MapCanvasTilesLayer};
use super::message::{MapEditorMessage, MapLayer, MapTool, MapViewMode, ObjectBrushMode};
use crate::app::App;
use crate::components::loading_state::LoadingState;
use crate::message::{Message, MessageExt};
use crate::style;
use gui_widgets::components::modal::modal;
use gui_widgets::components::toast;
use gui_widgets::lucide::{LUCIDE_FONT, icon_char};
use iced::widget::{
    button, canvas, column, container, progress_bar, row, stack, text, text_input, toggler,
};
use iced::{Element, Fill};
use lucide_icons::Icon;

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
            .into();
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

            let path_label = state
                .data
                .map_path
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            // ── Layer toggles, grouped into three segments ────────────────

            // Vertical toggle groups for the popover panel. Group headers live
            // in the panel itself (exactly one per group).
            let terrain_group = column![
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
            ]
            .spacing(6);

            let overlay_group = column![
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
                    "Object IDs",
                    state.view.show_object_ids,
                    tab_id,
                    MapLayer::ObjectIds,
                    None
                ),
            ]
            .spacing(6);

            let entity_group = column![
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
            .spacing(6);

            // ── Layers dropdown (popover) ─────────────────────────────────
            let layer_visibility = [
                state.view.show_ground,
                state.view.show_buildings,
                state.view.show_roofs,
                state.view.show_internal_sprites,
                state.view.show_collisions,
                state.view.show_events,
                state.view.show_monsters,
                state.view.show_npcs,
                state.view.show_npc_waypoints,
                state.view.show_objects,
                state.view.show_draw_items,
                state.view.show_object_ids,
            ];
            let visible_layers = layer_visibility.iter().filter(|&&v| v).count();
            let total_layers = layer_visibility.len();

            let layers_trigger = button(
                row![
                    text("LAYERS").size(11),
                    text("▾").size(10).style(style::subtle_text),
                    text(format!("· {}/{}", visible_layers, total_layers))
                        .size(10)
                        .style(style::subtle_text),
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center),
            )
            .padding([3, 10])
            .on_press(Message::map_editor(MapEditorMessage::ToggleLayersPopover(
                tab_id,
            )))
            .style(if state.view.layers_popover_open {
                style::active_chip
            } else {
                style::chip
            });

            let layers_panel = container(
                column![
                    segment_label("Terrain"),
                    terrain_group,
                    h_rule(),
                    segment_label("Overlays"),
                    overlay_group,
                    h_rule(),
                    segment_label("Entities"),
                    entity_group,
                ]
                .spacing(12),
            )
            .padding(14)
            .width(iced::Length::Fixed(200.0))
            .style(style::panel_container);

            let layers_popover: Element<'_, Message> = gui_widgets::components::popover::popover(
                layers_trigger,
                layers_panel,
                state.view.layers_popover_open,
                move || Message::map_editor(MapEditorMessage::ToggleLayersPopover(tab_id)),
                move || Message::map_editor(MapEditorMessage::CloseLayersPopover(tab_id)),
            );

            // ── Action buttons (Row B) ────────────────────────────────────
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

            // Icon + label undo/redo — icons alone are anti-UX.
            let mut undo_btn = button(
                row![
                    text(icon_char(Icon::Undo2)).font(LUCIDE_FONT).size(11),
                    text("Undo").size(11),
                ]
                .spacing(4),
            )
            .padding([3, 8]);
            if can_undo {
                undo_btn = undo_btn.on_press(Message::map_editor(MapEditorMessage::Undo(tab_id)));
            }

            let mut redo_btn = button(
                row![
                    text(icon_char(Icon::Redo2)).font(LUCIDE_FONT).size(11),
                    text("Redo").size(11),
                ]
                .spacing(4),
            )
            .padding([3, 8]);
            if can_redo {
                redo_btn = redo_btn.on_press(Message::map_editor(MapEditorMessage::Redo(tab_id)));
            }

            let export_label = if state.data.is_exporting {
                "Exporting…"
            } else {
                "Export PNG"
            };
            let mut export_btn = button(text(export_label).size(11))
                .padding([3, 8])
                .accessible_label("Export map as PNG");
            if !state.data.is_exporting {
                export_btn =
                    export_btn.on_press(Message::map_editor(MapEditorMessage::ExportImage(tab_id)));
            }

            let mut tmx_btn = button(text("Export TMX").size(11))
                .padding([3, 8])
                .accessible_label("Export map as TMX");
            if !state.data.is_exporting {
                tmx_btn =
                    tmx_btn.on_press(Message::map_editor(MapEditorMessage::ExportTmx(tab_id)));
            }

            // Folded summary replacing the duplicate Handles/Tiles info cells.
            let summary_text = format!(
                "{}×{} · {} NPC · {} gtl",
                model.tiled_map_width,
                model.tiled_map_height,
                state.data.npcs.len(),
                state.data.gtl_handles.len()
            );

            // Per-tool click hint. Hidden for Pan (default tool needs no help
            // text); always visible for editing tools.
            let hint: Option<String> = match state.view.active_tool {
                MapTool::Pan => None,
                MapTool::Collision => Some("Click tile: block/unblock".into()),
                MapTool::ObjectId => Some(match state.view.object_brush_mode {
                    ObjectBrushMode::Paint => {
                        format!("Click: paint obj {}", state.data.object_brush)
                    }
                    ObjectBrushMode::Erase => "Click: erase any value".into(),
                }),
                MapTool::EventInspect => Some("Click tile: inspect event".into()),
            };

            let actions_row = row![
                save_btn,
                undo_btn,
                redo_btn,
                export_btn,
                tmx_btn,
                text(path_label).size(10).style(style::subtle_text),
                horizontal_space(),
                match &hint {
                    Some(h) => Element::new(text(h.clone()).size(10).style(style::subtle_text)),
                    None => Element::new(horizontal_space()),
                },
            ]
            .spacing(6)
            .padding([4, 16])
            .align_y(iced::Alignment::Center);

            // ── Row A: context row ────────────────────────────────────────
            let mode_chip = |label: &'static str, mode: MapViewMode| {
                button(text(label).size(11))
                    .on_press(Message::map_editor(MapEditorMessage::SwitchViewMode(
                        tab_id, mode,
                    )))
                    .padding([3, 8])
                    .style(if state.view.view_mode == mode {
                        style::active_chip
                    } else {
                        style::chip
                    })
            };

            let tool_chip = |label: &'static str, tool: MapTool| {
                button(text(label).size(11))
                    .on_press(Message::map_editor(MapEditorMessage::SelectTool(
                        tab_id, tool,
                    )))
                    .padding([3, 8])
                    .style(if state.view.active_tool == tool {
                        style::active_chip
                    } else {
                        style::chip
                    })
            };

            // Contextual object-id brush options — inline in Row A, only when
            // the Obj ID tool is active.
            let brush_slot: Option<Element<'_, Message>> =
                if state.view.active_tool == MapTool::ObjectId {
                    let mode_chip = |label: &'static str, mode: ObjectBrushMode| {
                        button(text(label).size(11))
                            .on_press(Message::map_editor(MapEditorMessage::SetObjectBrushMode(
                                tab_id, mode,
                            )))
                            .padding([3, 6])
                            .style(if state.view.object_brush_mode == mode {
                                style::active_chip
                            } else {
                                style::chip
                            })
                    };
                    let brush = state.data.object_brush;
                    let preset_chip = |n: i32| {
                        button(text(n.to_string()).size(11))
                            .on_press(Message::map_editor(MapEditorMessage::SetObjectBrush(
                                tab_id, n,
                            )))
                            .padding([3, 5])
                            .style(style::chip)
                    };
                    let brush_str = brush.to_string();
                    Some(
                        row![
                            rule(),
                            mode_chip("Paint", ObjectBrushMode::Paint),
                            mode_chip("Erase", ObjectBrushMode::Erase),
                            rule(),
                            button(text("−").size(12))
                                .on_press(Message::map_editor(MapEditorMessage::SetObjectBrush(
                                    tab_id,
                                    brush - 1
                                )))
                                .padding([3, 6])
                                .style(style::chip),
                            container(
                                text_input("1", brush_str)
                                    .width(iced::Length::Fixed(44.0))
                                    .on_input(move |v: String| {
                                        let val = v.parse::<i32>().unwrap_or(1).clamp(1, 511);
                                        Message::map_editor(MapEditorMessage::SetObjectBrush(
                                            tab_id, val,
                                        ))
                                    }),
                            )
                            .padding([2, 4])
                            .style(style::info_card),
                            button(text("+").size(12))
                                .on_press(Message::map_editor(MapEditorMessage::SetObjectBrush(
                                    tab_id,
                                    brush + 1
                                )))
                                .padding([3, 6])
                                .style(style::chip),
                            preset_chip(1),
                            preset_chip(2),
                            preset_chip(3),
                            preset_chip(5),
                            preset_chip(10),
                        ]
                        .spacing(3)
                        .align_y(iced::Alignment::Center)
                        .into(),
                    )
                } else {
                    None
                };

            let mut context_row = row![].spacing(6).padding([4, 16]);
            context_row = context_row
                .push(mode_chip("Map", MapViewMode::Map))
                .push(mode_chip("Sprites", MapViewMode::Sprites))
                .push(rule())
                .push(tool_chip("Pan", MapTool::Pan))
                .push(tool_chip("Collide", MapTool::Collision))
                .push(tool_chip("Obj ID", MapTool::ObjectId))
                .push(tool_chip("Event", MapTool::EventInspect));
            if let Some(slot) = brush_slot {
                context_row = context_row.push(slot);
            }
            context_row = context_row
                .push(horizontal_space())
                .push(text(summary_text).size(10).style(style::subtle_text))
                .push(layers_popover);

            let toolbar = container(column![context_row, actions_row].spacing(0))
                .width(Fill)
                .style(style::toolbar_container)
                .accessible_label("Map editor toolbar");

            // ── Canvas for tile layers, sprites (images) ───────────────────────
            let tiles_canvas = canvas(MapCanvasTilesLayer { state })
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
            let root: Element<'_, Message> = if let Some(ref preview) = state.view.dialog_preview {
                modal(
                    base,
                    dialog_preview::view_dialog_preview(state, tab_id, preview),
                    move || Message::map_editor(MapEditorMessage::HideDialogPreview(tab_id)),
                    0.5,
                )
                .into()
            } else {
                base
            };

            // Toast notifications overlay (top-right, auto-dismiss).
            toast::Manager::new(root, &state.data.toasts, move |i| {
                Message::map_editor(MapEditorMessage::DismissToast(tab_id, i))
            })
            .timeout(3)
            .into()
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

use iced::widget::space::Space;

/// Fill remaining horizontal space (pushes following widgets right).
fn horizontal_space() -> Space {
    Space::new().width(Fill)
}

/// Thin vertical separator between layer-toggle segments.
fn rule() -> Element<'static, Message> {
    container(text(""))
        .width(iced::Length::Fixed(1.0))
        .height(iced::Length::Fixed(16.0))
        .style(|_| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.6, 0.55, 0.45, 0.4,
            ))),
            ..container::Style::default()
        })
        .into()
}

fn segment_label(label: &'static str) -> Element<'static, Message> {
    text(label.to_string())
        .size(10)
        .style(style::subtle_text)
        .into()
}

/// Thin horizontal separator between layer groups inside the popover panel.
fn h_rule() -> Element<'static, Message> {
    container(text(""))
        .width(Fill)
        .height(iced::Length::Fixed(1.0))
        .style(|_| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.6, 0.55, 0.45, 0.4,
            ))),
            ..container::Style::default()
        })
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
