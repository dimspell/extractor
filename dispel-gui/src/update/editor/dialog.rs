// Dialog editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages_tab;
use crate::message::editor::dialog::DialogEditorMessage;
use crate::message::MessageExt;
use dispel_core::Dialog;
use iced::Task;

pub fn handle(message: DialogEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = app
        .state
        .workspace
        .active()
        .map(|t| t.id)
        .unwrap_or(usize::MAX);

    match message {
        DialogEditorMessage::ScanDialogs => {
            if let Some(editor) = app.state.dialog_editors.get_mut(&tab_id) {
                editor.editor.loading_state = crate::loading_state::LoadingState::Loading;
                let path = editor.current_file.clone();
                return Task::perform(
                    async move {
                        let path = std::path::PathBuf::from(path);
                        // Use the trait method explicitly
                        let dialogs: std::io::Result<Vec<Dialog>> =
                            <Dialog as dispel_core::references::extractor::Extractor>::read_file(
                                &path,
                            );
                        dialogs.map_err(|e| e.to_string())
                    },
                    |res: Result<Vec<Dialog>, String>| {
                        <crate::message::Message as crate::message::MessageExt>::dialog(
                            DialogEditorMessage::Scanned(res),
                        )
                    },
                );
            }
            Task::none()
        }
        DialogEditorMessage::Scanned(result) => {
            if let Some(editor) = app.state.dialog_editors.get_mut(&tab_id) {
                editor.editor.loading_state = crate::loading_state::LoadingState::Loaded(());
                match result {
                    Ok(catalog) => {
                        editor.editor.status_msg =
                            format!("Dialog catalog loaded: {} entries", catalog.len());
                        editor.editor.catalog = Some(catalog);
                        editor.editor.refresh();
                        // Initialize spreadsheet
                        if let Some(spreadsheet) = app.state.dialog_spreadsheets.get_mut(&tab_id) {
                            spreadsheet.active = true;
                            spreadsheet.init_filter(editor.editor.catalog.as_ref().unwrap());
                            spreadsheet.compute_all_caches(editor.editor.catalog.as_ref().unwrap());
                            spreadsheet.init_pane_state();
                        }
                    }
                    Err(e) => {
                        editor.editor.status_msg = format!("Error loading dialog catalog: {}", e);
                    }
                }
            }
            Task::none()
        }
        DialogEditorMessage::SelectDialog(index) => {
            if let Some(editor) = app.state.dialog_editors.get_mut(&tab_id) {
                editor.select_dialog(index);
            }
            Task::none()
        }
        DialogEditorMessage::FieldChanged(index, field, value) => {
            if let Some(editor) = app.state.dialog_editors.get_mut(&tab_id) {
                editor.update_field(index, &field, value);
            }
            Task::none()
        }
        DialogEditorMessage::Save => {
            if let Some(editor) = app.state.dialog_editors.get_mut(&tab_id) {
                editor.editor.loading_state = crate::loading_state::LoadingState::Loading;
                let result = editor.save();
                editor.editor.loading_state = crate::loading_state::LoadingState::Loaded(());
                match result {
                    Ok(_) => editor.editor.status_msg = "Dialogs saved successfully.".into(),
                    Err(e) => editor.editor.status_msg = format!("Error saving dialogs: {}", e),
                }
            }
            Task::none()
        }
        DialogEditorMessage::Saved(_) => Task::none(),
        DialogEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages_tab!(
                app,
                dialog_spreadsheets,
                dialog_editors,
                &tab_id,
                |index, field, value| crate::message::Message::dialog(
                    DialogEditorMessage::FieldChanged(index, field, value)
                ),
                msg
            );
            Task::none()
        }
        DialogEditorMessage::PaneResized(event) => {
            if let Some(ed) = app.state.dialog_editors.get_mut(&tab_id) {
                if let Some(ref mut ps) = ed.editor.pane_state {
                    ps.resize(event.split, event.ratio);
                }
            }
            if let Some(ss) = app.state.dialog_spreadsheets.get_mut(&tab_id) {
                if let Some(ref mut ps) = ss.pane_state {
                    ps.resize(event.split, event.ratio);
                }
            }
            Task::none()
        }
        DialogEditorMessage::PaneClicked(pane) => {
            if let Some(ed) = app.state.dialog_editors.get_mut(&tab_id) {
                ed.editor.pane_focus = Some(pane);
            }
            Task::none()
        }
    }
}
