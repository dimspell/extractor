//! View rendering for the tile grid canvas view.

use super::canvas::TileGridCanvas;
use super::message::TileGridCanvasMessage;
use super::state::{PaneContent, TileGridViewState};
use crate::message::{Message, MessageExt};
use dispel_core::references::tile_grid::{TileDescriptor, TileGrid};
use iced::widget::{button, canvas, column, pane_grid, row, scrollable, text, text_input};
use iced::{Element, Length};

/// Render the tile grid canvas view: a resizable pane grid with the canvas
/// on the left and an inspector panel on the right.
pub fn view_canvas<'a>(grid: &'a TileGrid, view: &'a TileGridViewState) -> Element<'a, Message> {
    // The pane_grid widget needs a reference to a State. We fall back to
    // a transient local State if the persistent one is missing — the user
    // won't be able to resize in that case, but the layout still renders.
    let pane_state_ref: &pane_grid::State<PaneContent> = match view.pane_state.as_ref() {
        Some(s) => s,
        None => {
            // Leak a static State so the reference is valid for 'a.
            Box::leak(Box::new(pane_grid::State::new(PaneContent::Canvas).0))
        }
    };

    pane_grid::PaneGrid::new(pane_state_ref, |_pane, content, _is_maximized| {
        let inner: Element<Message> = match content {
            PaneContent::Canvas => canvas(TileGridCanvas { grid, view })
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
            PaneContent::Inspector => {
                let inspector = inspector_panel(grid, view);
                scrollable(inspector)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
        };
        // Wrap in `pane_grid::Content` so the pane_grid forwards mouse
        // events (ButtonPressed/CursorMoved/WheelScrolled) to the child
        // instead of capturing them for resize-handle logic. Content::new
        // already provides a container with `Length::Fill` sizing, so we
        // must NOT wrap the inner element in another container — double
        // wrapping collapses the layout to zero size.
        pane_grid::Content::new(inner)
    })
    .on_resize(10, |event| {
        Message::tile_grid_canvas(TileGridCanvasMessage::PaneResized(event))
    })
    .on_click(|pane| Message::tile_grid_canvas(TileGridCanvasMessage::PaneClicked(pane)))
    .into()
}

fn inspector_panel<'a>(grid: &'a TileGrid, view: &'a TileGridViewState) -> Element<'a, Message> {
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

        let fields: [(&str, &str, i32); 4] = [
            ("object_id", "Object ID (0–1023):", desc.object_id as i32),
            (
                "tile_index",
                "Tile Index (0–32767):",
                desc.tile_index as i32,
            ),
            (
                "render_flags",
                "Render Flags (0–3):",
                desc.render_flags as i32,
            ),
            ("reserved", "Reserved (0–31):", desc.reserved as i32),
        ];

        for (field, label, value) in fields {
            // Use the edit buffer if present, else the live value.
            // This prevents the "clamped value mid-edit" bug: typing "999"
            // shows "999" in the field, not "1023" until the user commits.
            let display = if let Some(idx) = view.selected_cell {
                view.edit_buffers
                    .get(&(idx, field.to_string()))
                    .cloned()
                    .unwrap_or_else(|| value.to_string())
            } else {
                value.to_string()
            };
            col = col.push(text(label).size(12));
            col = col.push(
                text_input("", display)
                    .on_input(move |s| {
                        Message::tile_grid_canvas(TileGridCanvasMessage::EditFieldBuffer {
                            field: field.to_string(),
                            value: s,
                        })
                    })
                    .on_submit(Message::tile_grid_canvas(
                        TileGridCanvasMessage::CommitFieldBuffer {
                            field: field.to_string(),
                        },
                    ))
                    .width(Length::Fill),
            );
        }
    } else {
        col = col.push(text("No cell selected. Click a cell to inspect."));
    }

    if let Some(s) = view.last_status.as_deref() {
        col = col.push(text(s).size(12));
    }

    col = col
        .push(text(format!("Zoom: {:.2}×", view.zoom)).size(12))
        .push(
            row![
                button("Fit").on_press(Message::tile_grid_canvas(
                    TileGridCanvasMessage::FitToWindow
                )),
                button("1:1").on_press(Message::tile_grid_canvas(TileGridCanvasMessage::SetZoom(
                    1.0
                ))),
            ]
            .spacing(4),
        );

    col.into()
}

fn word_from_desc(d: TileDescriptor) -> u32 {
    (d.object_id as u32 & 0x03ff)
        | ((d.tile_index as u32 & 0x7fff) << 10)
        | ((d.render_flags as u32 & 0x03) << 25)
        | ((d.reserved as u32 & 0x1f) << 27)
}
