//! View rendering for the tile grid canvas view.

use super::canvas::TileGridCanvas;
use super::message::TileGridCanvasMessage;
use super::state::TileGridViewState;
use crate::message::{Message, MessageExt};
use dispel_core::references::tile_grid::{TileDescriptor, TileGrid};
use iced::widget::{canvas, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

/// Render the tile grid canvas view: a pan/zoom canvas on the left,
/// an inspector panel on the right for editing the selected cell.
pub fn view_canvas<'a>(
    grid: &'a TileGrid,
    view: &'a TileGridViewState,
    status: Option<&'a str>,
) -> Element<'a, Message> {
    let canvas_widget = canvas(TileGridCanvas { grid, view })
        .width(Length::Fill)
        .height(Length::Fill);

    let inspector = inspector_panel(grid, view, status);

    row![
        container(canvas_widget)
            .width(Length::FillPortion(3))
            .height(Length::Fill),
        container(scrollable(inspector))
            .width(Length::FillPortion(1))
            .height(Length::Fill),
    ]
    .into()
}

fn inspector_panel<'a>(
    grid: &'a TileGrid,
    view: &TileGridViewState,
    status: Option<&'a str>,
) -> Element<'a, Message> {
    let mut col = column![text("Inspector").size(18)].spacing(6).padding(10);

    if let Some(idx) = view.selected_cell {
        let gx = idx % grid.grid_size;
        let gy = idx / grid.grid_size;
        let desc = grid
            .data
            .get(idx as usize)
            .copied()
            .map(TileDescriptor::from_word)
            .unwrap_or(TileDescriptor {
                object_id: 0,
                tile_index: 0,
                render_flags: 0,
                reserved: 0,
            });

        col = col
            .push(text(format!("Cell index: {idx}")))
            .push(text(format!("Grid coords: ({gx}, {gy})")))
            .push(text(format!("Raw word: 0x{:08X}", word_from_desc(desc))));

        let fields: [(&str, i32); 4] = [
            ("Object ID (0–1023):", desc.object_id as i32),
            ("Tile Index (0–32767):", desc.tile_index as i32),
            ("Render Flags (0–3):", desc.render_flags as i32),
            ("Reserved (0–31):", desc.reserved as i32),
        ];

        for (label, value) in fields {
            let d = desc;
            let value_str = value.to_string();
            col = col.push(text(label).size(12));
            col = col.push(
                text_input("", value_str)
                    .on_input(move |s| {
                        if let Ok(v) = s.parse::<i32>() {
                            let mut new_desc = d;
                            apply_field(&mut new_desc, label, v);
                            Message::tile_grid_canvas(TileGridCanvasMessage::EditSelected(new_desc))
                        } else {
                            Message::tile_grid_canvas(TileGridCanvasMessage::ClearSelection)
                        }
                    })
                    .width(Length::Fill),
            );
        }
    } else {
        col = col.push(text("No cell selected. Click a cell to inspect."));
    }

    if let Some(s) = status {
        col = col.push(text(s).size(12));
    }

    col.into()
}

fn word_from_desc(d: TileDescriptor) -> u32 {
    (d.object_id as u32 & 0x03ff)
        | ((d.tile_index as u32 & 0x7fff) << 10)
        | ((d.render_flags as u32 & 0x03) << 25)
        | ((d.reserved as u32 & 0x1f) << 27)
}

/// Apply a clamped value to the correct field of a `TileDescriptor` based on the label.
fn apply_field(d: &mut TileDescriptor, label: &str, v: i32) {
    if label.starts_with("Object ID") {
        d.object_id = v.clamp(0, 1023) as u16;
    } else if label.starts_with("Tile Index") {
        d.tile_index = v.clamp(0, 32767) as u16;
    } else if label.starts_with("Render Flags") {
        d.render_flags = v.clamp(0, 3) as u8;
    } else if label.starts_with("Reserved") {
        d.reserved = v.clamp(0, 31) as u8;
    }
}
