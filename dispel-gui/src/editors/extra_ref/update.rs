use dispel_core::Extractor;
use iced::Task;

use crate::app::App;
use crate::components::editable::EditableRecord;
use crate::editors::extra_ref::ExtraRefEditorMessage;
use crate::handle_spreadsheet_messages_tab;
use crate::components::loading_state::LoadingState;
use crate::message::MessageExt;
use crate::update::editor::tab;

pub fn handle(msg: ExtraRefEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = tab::get_tab_id(&app.state.workspace);

    match msg {
        ExtraRefEditorMessage::LoadCatalog(path) => {
            tab::load_catalog_sync(
                path.clone(),
                &mut app.state.extra_ref_editors,
                &mut app.state.extra_ref_spreadsheets,
                tab_id,
            );
            Task::perform(
                async move {
                    dispel_core::ExtraRef::read_file(&path)
                        .map_err(|e: std::io::Error| e.to_string())
                },
                move |result| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::ExtraRef(
                            ExtraRefEditorMessage::CatalogLoaded(tab_id, result),
                        ),
                    )
                },
            )
        }
        ExtraRefEditorMessage::CatalogLoaded(id, result) => {
            if let Some(editor) = app.state.extra_ref_editors.get_mut(&id) {
                editor.editor.loading_state = LoadingState::Loaded(());
                match result {
                    Ok(catalog) => {
                        editor.editor.status_msg =
                            format!("Extra ref catalog loaded: {} entries", catalog.len());
                        editor.editor.catalog = Some(catalog.clone());
                        if let Some(ss) = app.state.extra_ref_spreadsheets.get_mut(&id) {
                            ss.apply_filter(&catalog);
                            ss.compute_all_caches(&catalog);
                            ss.init_pane_state();
                        }
                    }
                    Err(e) => {
                        editor.editor.status_msg =
                            format!("Error loading extra ref catalog: {}", e);
                    }
                }
            }
            Task::none()
        }
        ExtraRefEditorMessage::Select(index) => {
            tab::select(&mut app.state.extra_ref_editors, tab_id, index)
        }
         ExtraRefEditorMessage::FieldChanged(index, field, value) => {
             let (old_value, orig_idx, file_path) = app
                 .state
                 .extra_ref_editors
                 .get(&tab_id)
                 .map(|e| {
                     let (oidx, old) = e
                         .editor
                         .filtered
                         .get(index)
                         .map(|(i, r)| (*i as u32, r.get_field(&field)))
                         .unwrap_or((0, String::new()));
                     let fp = e.current_file.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
                     (old, oidx, fp)
                 })
                 .unwrap_or_default();
             let new_value = value.clone();
             let task = tab::field_changed(
                 &mut app.state.extra_ref_editors,
                 tab_id,
                 index,
                 field.clone(),
                 value,
             );
             let observe = if old_value != new_value {
                 crate::editors::mod_packager::recording::observe_field_change(
                     app, file_path, orig_idx, &field, old_value, new_value,
                 )
             } else {
                 iced::Task::none()
             };
             observe.chain(task)
         }
        ExtraRefEditorMessage::Save => tab::save(
            &mut app.state.extra_ref_editors,
            tab_id,
            "Extra refs saved successfully.",
            "Error saving extra refs",
        ),
        ExtraRefEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages_tab!(
                app,
                extra_ref_spreadsheets,
                extra_ref_editors,
                &tab_id,
                |index, field, value| crate::message::Message::extra_ref(
                    ExtraRefEditorMessage::FieldChanged(index, field, value)
                ),
                msg
            );
            Task::none()
        }
        ExtraRefEditorMessage::PaneResized(event) => tab::pane_resized(
            &mut app.state.extra_ref_editors,
            &mut app.state.extra_ref_spreadsheets,
            tab_id,
            event,
        ),
        ExtraRefEditorMessage::PaneClicked(pane) => {
            tab::pane_clicked(&mut app.state.extra_ref_editors, tab_id, pane)
        }
    }
}
