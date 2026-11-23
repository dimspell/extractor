use crate::app::App;
use crate::editors::bmp_viewer::{BmpViewerMessage, fit_scale};
use crate::message::{Message, MessageExt};
use gui_widgets::lucide::{LUCIDE_FONT, icon_char};
use iced::widget::{Space, button, column, container, image, responsive, row, scrollable, text};
use iced::{Alignment, Element, Fill, Size};
use lucide_icons::Icon;

pub fn view(app: &App) -> Element<'_, Message> {
    let tab_id = app.state.workspace.active_tab_id().as_usize();

    let Some(viewer) = app.state.editors.bmp_viewers.get(&tab_id) else {
        return container(text("Image not loaded").size(14))
            .width(Fill)
            .height(Fill)
            .padding(16)
            .accessible_label("BMP viewer")
            .into();
    };

    if let Some(ref err) = viewer.error {
        return container(
            column![text("Failed to load image").size(14), text(err).size(12)].spacing(8),
        )
        .width(Fill)
        .height(Fill)
        .padding(16)
        .accessible_label("BMP viewer")
        .into();
    }

    let Some(handle) = viewer.handle.clone() else {
        return container(text("Loading…").size(12))
            .width(Fill)
            .height(Fill)
            .accessible_label("BMP viewer")
            .into();
    };

    let zoom_button = |icon: Icon, label: &'static str, msg: BmpViewerMessage| {
        button(
            row![
                text(icon_char(icon)).font(LUCIDE_FONT).size(12),
                text(label).size(12),
            ]
            .spacing(4),
        )
        .on_press(Message::bmp_viewer(msg))
    };

    let header = row![
        text(&viewer.name).size(13),
        text(format!("{} × {}", viewer.width, viewer.height)).size(11),
        Space::new().width(Fill),
        zoom_button(Icon::ZoomOut, " −", BmpViewerMessage::ZoomOut),
        zoom_button(Icon::ZoomIn, " +", BmpViewerMessage::ZoomIn),
        zoom_button(Icon::RotateCcw, " Reset", BmpViewerMessage::ZoomReset),
        zoom_button(Icon::Maximize, " Fit", BmpViewerMessage::Fit),
    ]
    .spacing(6)
    .padding([6, 12])
    .align_y(Alignment::Center);

    let zoom_label =
        container(text(format!("Zoom {:.0}%", viewer.zoom * 100.0)).size(11)).padding([2, 12]);

    let fit_mode = viewer.fit_mode;
    let zoom = viewer.zoom;
    let width = viewer.width;
    let height = viewer.height;

    let canvas: Element<'_, Message> = responsive(move |size: Size| {
        let fit = if fit_mode {
            fit_scale(size, (width, height))
        } else {
            1.0
        };
        let scale = fit * zoom;
        let draw_w = (width as f32 * scale).max(1.0);
        let draw_h = (height as f32 * scale).max(1.0);

        let result: Element<'_, Message> =
            container(image(handle.clone()).width(draw_w).height(draw_h))
                .width(Fill)
                .height(Fill)
                .center_x(Fill)
                .center_y(Fill)
                .into();
        result
    })
    .width(Fill)
    .height(Fill)
    .into();

    column![
        header,
        zoom_label,
        scrollable(canvas).width(Fill).height(Fill)
    ]
    .spacing(0)
    .height(Fill)
    .accessible_label("BMP viewer")
    .into()
}
