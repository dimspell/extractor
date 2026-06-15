use crate::app::App;
use crate::editors::map_editor::{MapEditorMessage, SpriteExportDialogState, SpriteExportStatus};
use crate::message::{Message, MessageExt};
use iced::Task;
use std::path::PathBuf;

pub fn show_dialog(app: &mut App, tab_id: usize) -> Task<Message> {
    if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
        state.data.sprite_export_dialog = Some(SpriteExportDialogState::default());
    }
    Task::none()
}

pub fn close_dialog(app: &mut App, tab_id: usize) -> Task<Message> {
    if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
        state.data.sprite_export_dialog = None;
    }
    Task::none()
}

pub fn choose_dir(tab_id: usize) -> Task<Message> {
    Task::perform(
        async {
            rfd::AsyncFileDialog::new()
                .pick_folder()
                .await
                .map(|h| h.path().to_path_buf())
        },
        move |path| Message::map_editor(MapEditorMessage::SpriteExportDirChosen(tab_id, path)),
    )
}

pub fn dir_chosen(app: &mut App, tab_id: usize, path: Option<PathBuf>) -> Task<Message> {
    if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
        if let Some(ref mut dlg) = state.data.sprite_export_dialog {
            dlg.export_dir = path;
            dlg.status = SpriteExportStatus::Idle;
        }
    }
    Task::none()
}

pub fn confirm_export(app: &mut App, tab_id: usize) -> Task<Message> {
    let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) else {
        return Task::none();
    };
    let Some(ref dlg) = state.data.sprite_export_dialog else {
        return Task::none();
    };
    let Some(ref export_dir) = dlg.export_dir else {
        return Task::none();
    };
    let Some(ref map_path) = state.data.map_path else {
        return Task::none();
    };
    let map_path = map_path.clone();
    let export_dir = export_dir.clone();

    if let Some(ref mut dlg) = state.data.sprite_export_dialog {
        dlg.status = SpriteExportStatus::Exporting;
    }

    Task::perform(
        async move {
            dispel_core::map::extract_sprites(&map_path, &export_dir)
                .map(|()| format!("Sprites exported → {}", export_dir.display()))
                .map_err(|e| e.to_string())
        },
        move |result| Message::map_editor(MapEditorMessage::SpriteExportDone(tab_id, result)),
    )
}

pub fn export_done(app: &mut App, tab_id: usize, result: Result<String, String>) -> Task<Message> {
    if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
        if let Some(ref mut dlg) = state.data.sprite_export_dialog {
            dlg.status = match result {
                Ok(msg) => SpriteExportStatus::Done(msg),
                Err(e) => SpriteExportStatus::Error(e),
            };
        }
    }
    Task::none()
}
