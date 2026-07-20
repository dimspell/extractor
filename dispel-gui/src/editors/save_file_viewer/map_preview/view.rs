//! View function for the map preview component.

use crate::components::map_render::GenericTilesLayer;
use crate::editors::save_file_viewer::map_preview::message::PreviewMessage;
use crate::editors::save_file_viewer::map_preview::overlay::MapPreviewOverlaysLayer;
use crate::editors::save_file_viewer::map_preview::state::{
    MapPreviewLoading, MapPreviewState, PreviewLayer,
};
use crate::message::Message;
use iced::widget::{button, canvas, column, container, progress_bar, row, stack, text};
use iced::{Element, Fill};

/// Render the map preview control panel and canvas.
pub fn view<'a>(state: &'a MapPreviewState) -> Element<'a, Message> {
    match &state.loading {
        MapPreviewLoading::Idle => container(text("Map preview: click a map to load"))
            .padding(16)
            .width(Fill)
            .height(Fill)
            .into(),

        MapPreviewLoading::Loading => container(
            column![text("Loading map…").size(12), progress_bar(0.0..=1.0, 0.5),]
                .spacing(8)
                .padding(16),
        )
        .width(Fill)
        .height(Fill)
        .into(),

        MapPreviewLoading::Failed(err) => container(
            column![
                text("Failed to load map preview")
                    .size(13)
                    .color(iced::Color::from_rgb(0.8, 0.2, 0.2)),
                text(err.as_str())
                    .size(11)
                    .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            ]
            .spacing(8)
            .padding(16),
        )
        .width(Fill)
        .height(Fill)
        .into(),

        MapPreviewLoading::Loaded => {
            if !state.tiles_ready {
                return container(
                    column![
                        text("Decoding tiles…").size(12),
                        progress_bar(0.0..=1.0, 0.5),
                    ]
                    .spacing(8)
                    .padding(16),
                )
                .width(Fill)
                .height(Fill)
                .into();
            }

            let layer_row = row![
                text("Layers:").size(11),
                layer_toggle("Ground", state.view.show_ground, PreviewLayer::Ground),
                layer_toggle(
                    "Buildings",
                    state.view.show_buildings,
                    PreviewLayer::Buildings
                ),
                layer_toggle("Roofs", state.view.show_roofs, PreviewLayer::Roofs),
                layer_toggle(
                    "Sprites",
                    state.view.show_internal_sprites,
                    PreviewLayer::InternalSprites
                ),
                layer_toggle("Monsters", state.view.show_monsters, PreviewLayer::Monsters),
                layer_toggle("NPCs", state.view.show_npcs, PreviewLayer::Npcs),
                layer_toggle("Extras", state.view.show_objects, PreviewLayer::Extras),
                layer_toggle("Items", state.view.show_draw_items, PreviewLayer::DrawItems),
            ]
            .spacing(12)
            .padding([6, 16])
            .align_y(iced::Alignment::Center);

            let zoom_label = format!("{:.0}%", state.view.zoom * 100.0);

            let zoom_controls = container(
                column![
                    button(text("+").size(14))
                        .on_press(Message::MapPreview(PreviewMessage::Zoom(
                            1.25,
                            f32::NAN,
                            f32::NAN
                        )))
                        .padding([5, 10]),
                    text(zoom_label).size(10),
                    button(text("−").size(14))
                        .on_press(Message::MapPreview(PreviewMessage::Zoom(
                            1.0 / 1.25,
                            f32::NAN,
                            f32::NAN
                        )))
                        .padding([5, 10]),
                    button(text("⊡").size(11))
                        .on_press(Message::MapPreview(PreviewMessage::FitToWindow))
                        .padding([5, 10]),
                ]
                .spacing(4)
                .align_x(iced::Alignment::Center),
            )
            .padding(8)
            .width(Fill)
            .height(Fill)
            .align_x(iced::alignment::Horizontal::Right)
            .align_y(iced::alignment::Vertical::Top);

            let tiles = canvas(GenericTilesLayer { state }).width(Fill).height(Fill);
            let overlays = canvas(MapPreviewOverlaysLayer { state })
                .width(Fill)
                .height(Fill);
            let map_stack = stack![tiles, overlays].width(Fill).height(Fill);
            let canvas_with_overlay = stack![map_stack, zoom_controls].width(Fill).height(Fill);

            column![layer_row, canvas_with_overlay]
                .spacing(0)
                .width(Fill)
                .height(Fill)
                .into()
        }
    }
}

fn layer_toggle(
    label: &'static str,
    is_on: bool,
    layer: PreviewLayer,
) -> Element<'static, Message> {
    iced::widget::toggler(is_on)
        .label(label)
        .text_size(11)
        .size(12)
        .on_toggle(move |_| Message::MapPreview(PreviewMessage::LayerToggle(layer)))
        .into()
}
