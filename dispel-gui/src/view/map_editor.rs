use crate::app::App;
use crate::components::editor::editable::{EditableRecord, FieldKind};
use crate::components::map_canvas::{MapCanvasOverlaysLayer, MapCanvasTilesLayer};
use crate::components::modal::modal;
use crate::loading_state::LoadingState;
use crate::message::editor::map_editor::{MapEditorMessage, MapLayer, MapViewMode, SelectedEntity};
use crate::message::{Message, MessageExt};
use crate::state::map_editor::{SpriteExportDialogState, SpriteExportStatus};
use crate::style;
use crate::utils::{horizontal_rule, horizontal_space};
use dispel_core::{ExtraRef, MonsterRef, NPC};
use iced::widget::{
    button, canvas, column, container, pick_list, progress_bar, row, scrollable, stack, text,
    text_input, toggler,
};
use iced::{Element, Fill, Font};

impl App {
    pub fn view_map_editor_tab(&self) -> Element<'_, Message> {
        let tab_id = match self.state.workspace.active() {
            Some(tab) => tab.id,
            None => return text("No active tab").into(),
        };

        let state = match self.state.map_editors.get(&tab_id) {
            Some(s) => s,
            None => {
                return container(
                    text("Map editor not initialised — reopen the file.")
                        .size(12)
                        .style(style::subtle_text),
                )
                .padding(24)
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
                            "{}M {}N {}O",
                            state.data.monsters.len(),
                            state.data.npcs.len(),
                            state.data.extra_refs.len()
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
                        "Objects",
                        state.view.show_objects,
                        tab_id,
                        MapLayer::Objects,
                        Some(state.data.extra_refs.len())
                    ),
                ]
                .spacing(12)
                .padding([6, 16])
                .align_y(iced::Alignment::Center);

                // ── Action buttons row ─────────────────────────────────────────
                let can_undo = !state.data.undo_stack.is_empty();
                let can_redo = !state.data.redo_stack.is_empty();
                let has_entity_files = state.data.monster_ref_path.is_some()
                    || state.data.npc_ref_path.is_some()
                    || state.data.extra_ref_path.is_some();

                let save_label = if state.data.is_saving {
                    "Saving…"
                } else if state.data.dirty {
                    "Save*"
                } else {
                    "Save"
                };
                let mut save_btn = button(text(save_label).size(11)).padding([3, 8]);
                if state.data.dirty && has_entity_files && !state.data.is_saving {
                    save_btn = save_btn
                        .on_press(Message::map_editor(MapEditorMessage::SaveEntities(tab_id)));
                }

                let mut undo_btn = button(text("↩ Undo").size(11)).padding([3, 8]);
                if can_undo {
                    undo_btn =
                        undo_btn.on_press(Message::map_editor(MapEditorMessage::Undo(tab_id)));
                }

                let mut redo_btn = button(text("↪ Redo").size(11)).padding([3, 8]);
                if can_redo {
                    redo_btn =
                        redo_btn.on_press(Message::map_editor(MapEditorMessage::Redo(tab_id)));
                }

                let export_label = if state.data.is_exporting {
                    "Exporting…"
                } else {
                    "Export PNG"
                };
                let mut export_btn = button(text(export_label).size(11)).padding([3, 8]);
                if !state.data.is_exporting {
                    export_btn = export_btn
                        .on_press(Message::map_editor(MapEditorMessage::ExportImage(tab_id)));
                }

                let status_text = if let Some(msg) = &state.data.status_msg {
                    text(msg.as_str()).size(10).style(style::subtle_text)
                } else {
                    text("").size(10).style(style::subtle_text)
                };

                let action_row = row![save_btn, undo_btn, redo_btn, export_btn, status_text,]
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
                        mode_tab_row,
                        row![info_row].spacing(0),
                        row![layer_row, tile_status]
                            .spacing(16)
                            .align_y(iced::Alignment::Center),
                        row![action_row].spacing(0),
                    ]
                    .spacing(0),
                )
                .width(Fill)
                .style(style::toolbar_container);

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
                        text(format!("{:.0}%", state.view.zoom * 100.0))
                            .size(10)
                            .style(style::subtle_text),
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

                let canvas_with_overlay =
                    stack![map_canvas, zoom_controls].width(Fill).height(Fill);

                // ── Body: map canvas or sprite browser ───────────────────────
                let body: Element<'_, Message> = match state.view.view_mode {
                    MapViewMode::Map => match state.view.selected_entity {
                        Some(sel) => {
                            let inspector = build_inspector(state, tab_id, sel);
                            row![canvas_with_overlay, inspector]
                                .width(Fill)
                                .height(Fill)
                                .into()
                        }
                        None => canvas_with_overlay.into(),
                    },
                    MapViewMode::Sprites => view_sprite_browser(state, tab_id),
                };

                column![toolbar, body]
                    .spacing(0)
                    .width(Fill)
                    .height(Fill)
                    .into()
            }
        }
    }
}

// ── Inspector ─────────────────────────────────────────────────────────────────

fn build_inspector<'a>(
    state: &'a crate::state::map_editor::MapEditorState,
    tab_id: usize,
    sel: SelectedEntity,
) -> Element<'a, Message> {
    let close_msg = Message::map_editor(MapEditorMessage::Deselect(tab_id));

    let (title, width, body): (&'static str, f32, Element<'a, Message>) = match sel {
        SelectedEntity::Monster(i) => {
            let body = if let Some(record) = state.data.monsters.get(i) {
                build_record_fields::<MonsterRef>(record, tab_id, sel)
            } else {
                text("Monster not found").size(12).into()
            };
            (MonsterRef::detail_title(), MonsterRef::detail_width(), body)
        }
        SelectedEntity::Npc(i) => {
            let body = if let Some(record) = state.data.npcs.get(i) {
                build_record_fields::<NPC>(record, tab_id, sel)
            } else {
                text("NPC not found").size(12).into()
            };
            (NPC::detail_title(), NPC::detail_width(), body)
        }
        SelectedEntity::Extra(i) => {
            let body = if let Some(record) = state.data.extra_refs.get(i) {
                build_record_fields::<ExtraRef>(record, tab_id, sel)
            } else {
                text("Object not found").size(12).into()
            };
            (ExtraRef::detail_title(), ExtraRef::detail_width(), body)
        }
    };

    let header = row![
        text(title)
            .size(12)
            .font(Font::MONOSPACE)
            .style(style::subtle_text),
        horizontal_space(),
        button(text("×").size(14))
            .on_press(close_msg)
            .padding([3, 8])
            .style(style::browse_button),
    ]
    .align_y(iced::Alignment::Center)
    .padding([0, 4]);

    container(
        column![header, horizontal_rule(1), body]
            .spacing(6)
            .padding(10),
    )
    .width(width)
    .height(Fill)
    .style(style::inspector_container)
    .into()
}

/// Iterate all `FieldDescriptor`s for `R` and build a scrollable column of editor rows.
///
/// `text_input` copies its value string internally, so the `String` returned by
/// `get_field` can be a temporary — the resulting `Element` has no lifetime tie to it.
fn build_record_fields<'a, R: EditableRecord>(
    record: &R,
    tab_id: usize,
    sel: SelectedEntity,
) -> Element<'a, Message> {
    let mut col = column![].spacing(5);
    for desc in R::field_descriptors() {
        let value = record.get_field(desc.name);
        col = col.push(inspector_field_row(
            desc.label, desc.name, &desc.kind, &value, tab_id, sel,
        ));
    }
    scrollable(col).into()
}

/// Render a single labeled field row for the map editor inspector.
///
/// `label` and `name` are `&'static str` (from `FieldDescriptor`); `value` is a
/// short-lived borrow of a locally-computed `String` — safe because `text_input`
/// and `pick_list` copy their value arguments before returning the widget.
fn inspector_field_row<'a>(
    label: &'static str,
    name: &'static str,
    kind: &FieldKind,
    value: &str,
    tab_id: usize,
    sel: SelectedEntity,
) -> Element<'a, Message> {
    const LABEL_W: f32 = 140.0;
    match kind {
        FieldKind::String
        | FieldKind::TextArea
        | FieldKind::Integer
        | FieldKind::Boolean
        | FieldKind::Lookup(_) => row![
            text(label)
                .size(11)
                .width(LABEL_W)
                .style(style::subtle_text),
            text_input("", value)
                .on_input(move |v| {
                    Message::map_editor(MapEditorMessage::EntityFieldChanged(
                        tab_id,
                        sel,
                        name.to_string(),
                        v,
                    ))
                })
                .padding(4)
                .size(11),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center)
        .into(),

        FieldKind::Enum { variants } => {
            let options: Vec<&'static str> = variants.to_vec();
            let selected = options
                .iter()
                .find(|&&opt| opt == value)
                .copied()
                .or_else(|| options.first().copied());
            row![
                text(label)
                    .size(11)
                    .width(LABEL_W)
                    .style(style::subtle_text),
                pick_list(options, selected, move |v: &'static str| {
                    Message::map_editor(MapEditorMessage::EntityFieldChanged(
                        tab_id,
                        sel,
                        name.to_string(),
                        v.to_string(),
                    ))
                })
                .padding(4)
                .text_size(11),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center)
            .into()
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
        .size(12)
        .on_toggle(move |_| Message::map_editor(MapEditorMessage::LayerToggled(tab_id, layer)))
        .into()
}

// ── Sprite browser ────────────────────────────────────────────────────────────

fn view_sprite_browser<'a>(
    state: &'a crate::state::map_editor::MapEditorState,
    tab_id: usize,
) -> Element<'a, Message> {
    use iced::widget::image;
    use iced::Length::Fixed;

    let handles = &state.data.sprite_sequence_handles;

    if handles.is_empty() {
        return container(
            text("No embedded sprites in this map.")
                .size(12)
                .style(style::subtle_text),
        )
        .padding(24)
        .into();
    }

    let selected = state.view.selected_sprite_sequence;

    let thumbnails: Vec<Element<'_, Message>> = handles
        .iter()
        .map(|s| {
            let thumb = column![
                image(s.handle.clone())
                    .width(Fixed(64.0))
                    .height(Fixed(64.0)),
                text(format!("#{}", s.sequence_idx)).size(10),
                text(format!("{}×{}", s.width, s.height))
                    .size(10)
                    .style(style::subtle_text),
                text(format!("×{} placed", s.placement_count))
                    .size(10)
                    .style(style::subtle_text),
            ]
            .spacing(2)
            .align_x(iced::Alignment::Center);

            let is_selected = selected == Some(s.sequence_idx);
            button(thumb)
                .on_press(Message::map_editor(MapEditorMessage::SelectSpriteSequence(
                    tab_id,
                    if is_selected {
                        None
                    } else {
                        Some(s.sequence_idx)
                    },
                )))
                .padding(6)
                .style(if is_selected {
                    style::active_chip
                } else {
                    style::chip
                })
                .into()
        })
        .collect();

    let header = row![
        text(format!(
            "{} sprite sequence{}",
            handles.len(),
            if handles.len() == 1 { "" } else { "s" }
        ))
        .size(11)
        .style(style::subtle_text),
        horizontal_space(),
        button(text("Export…").size(11))
            .on_press(Message::map_editor(
                MapEditorMessage::ShowSpriteExportDialog(tab_id)
            ))
            .padding([3, 8])
            .style(style::export_button),
    ]
    .padding([4, 16])
    .align_y(iced::Alignment::Center)
    .width(Fill);

    let grid: Element<'_, Message> = scrollable(
        column![
            text("Sprites").size(11).style(style::subtle_text),
            row(thumbnails).spacing(8).padding([8, 16]).wrap(),
        ]
        .spacing(4),
    )
    .width(Fill)
    .height(Fill)
    .into();

    let detail: Element<'_, Message> = if let Some(idx) = selected {
        if let Some(s) = handles.iter().find(|h| h.sequence_idx == idx) {
            let placement_items: Vec<Element<'_, Message>> = s
                .placements
                .iter()
                .map(|(x, y)| text(format!("  ({x}, {y})")).size(11).into())
                .collect();

            scrollable(
                column![
                    text(format!(
                        "Sprite #{} — {}×{}px — {} placement{}",
                        s.sequence_idx,
                        s.width,
                        s.height,
                        s.placement_count,
                        if s.placement_count == 1 { "" } else { "s" },
                    ))
                    .size(12),
                    column(placement_items).spacing(2),
                ]
                .spacing(8)
                .padding([8, 16]),
            )
            .width(Fill)
            .height(Fill)
            .into()
        } else {
            text("").into()
        }
    } else {
        container(
            text("Select a sprite to see placements.")
                .size(11)
                .style(style::subtle_text),
        )
        .padding([8, 16])
        .into()
    };

    // Split pane: grid on left (70%), detail on right (30%)
    let split_content: Element<'_, Message> = row![
        container(grid)
            .width(iced::Length::FillPortion(7))
            .height(Fill),
        container(detail)
            .width(iced::Length::FillPortion(3))
            .height(Fill),
    ]
    .width(Fill)
    .height(Fill)
    .spacing(0)
    .into();

    let base: Element<'_, Message> = column![header, split_content]
        .spacing(0)
        .width(Fill)
        .height(Fill)
        .into();

    if let Some(ref dlg) = state.data.sprite_export_dialog {
        modal(
            base,
            view_sprite_export_dialog(dlg, tab_id),
            move || Message::map_editor(MapEditorMessage::CloseSpriteExportDialog(tab_id)),
            0.5,
        )
    } else {
        base
    }
}

fn view_sprite_export_dialog<'a>(
    dlg: &'a SpriteExportDialogState,
    tab_id: usize,
) -> Element<'a, Message> {
    let title = text("Export Map Sprites")
        .size(14)
        .style(style::primary_text);

    let dir_label = if let Some(ref p) = dlg.export_dir {
        text(p.display().to_string()).size(11)
    } else {
        text("No folder selected")
            .size(11)
            .style(style::subtle_text)
    };

    let choose_btn = button(text("Choose Folder…").size(11))
        .on_press(Message::map_editor(
            MapEditorMessage::ChooseSpriteExportDir(tab_id),
        ))
        .padding([4, 10]);

    let can_export = dlg.export_dir.is_some() && dlg.status != SpriteExportStatus::Exporting;

    let export_btn = if can_export {
        button(text("Export").size(12))
            .on_press(Message::map_editor(MapEditorMessage::ConfirmSpriteExport(
                tab_id,
            )))
            .padding([5, 16])
            .style(style::export_button)
    } else {
        button(text("Export").size(12)).padding([5, 16])
    };

    let cancel_btn = button(text("Cancel").size(12))
        .on_press(Message::map_editor(
            MapEditorMessage::CloseSpriteExportDialog(tab_id),
        ))
        .padding([5, 16]);

    let status_row: Element<'_, Message> = match &dlg.status {
        SpriteExportStatus::Idle => text("").size(11).into(),
        SpriteExportStatus::Exporting => {
            text("Exporting…").size(11).style(style::subtle_text).into()
        }
        SpriteExportStatus::Done(msg) => text(msg.as_str()).size(11).into(),
        SpriteExportStatus::Error(e) => text(e.as_str())
            .size(11)
            .color(iced::Color::from_rgb(0.8, 0.2, 0.2))
            .into(),
    };

    container(
        column![
            title,
            horizontal_rule(1),
            text("Output folder:").size(11).style(style::subtle_text),
            row![dir_label, horizontal_space(), choose_btn]
                .align_y(iced::Alignment::Center)
                .spacing(8),
            horizontal_rule(1),
            status_row,
            row![cancel_btn, export_btn].spacing(8),
        ]
        .spacing(12)
        .padding(20)
        .width(iced::Length::Fixed(400.0)),
    )
    .style(style::toolbar_container)
    .into()
}
