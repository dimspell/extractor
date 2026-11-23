use crate::app::App;
use crate::editors::bmp_viewer::{
    BmpViewerMessage, MAX_ZOOM, MIN_ZOOM, ZOOM_STEP,
};
use iced::Task;

pub fn handle(message: BmpViewerMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = app.state.workspace.active_tab_id().as_usize();
    let Some(viewer) = app.state.editors.bmp_viewers.get_mut(&tab_id) else {
        return Task::none();
    };

    match message {
        BmpViewerMessage::ZoomIn => {
            viewer.zoom = (viewer.zoom * ZOOM_STEP).clamp(MIN_ZOOM, MAX_ZOOM);
            viewer.fit_mode = false;
        }
        BmpViewerMessage::ZoomOut => {
            viewer.zoom = (viewer.zoom / ZOOM_STEP).clamp(MIN_ZOOM, MAX_ZOOM);
            viewer.fit_mode = false;
        }
        BmpViewerMessage::ZoomReset => {
            viewer.zoom = 1.0;
            viewer.fit_mode = false;
        }
        BmpViewerMessage::Fit => {
            viewer.zoom = 1.0;
            viewer.fit_mode = true;
        }
    }

    Task::none()
}
