use crate::app::App;
use crate::editors::map_editor::{DialogPreviewState, MapEditorMessage};
use crate::message::{Message, MessageExt};
use dispel_core::references::extractor::Extractor;
use iced::Task;

pub fn show_preview(app: &mut App, tab_id: usize, npc_idx: usize) -> Task<Message> {
    let state = match app.state.editors.map_editors.get(&tab_id) {
        Some(s) => s,
        None => return Task::none(),
    };
    let game_path = match &app.state.workspace.game_path {
        Some(p) => p.clone(),
        None => return Task::none(),
    };
    let map_path = match &state.data.map_path {
        Some(p) => p.clone(),
        None => return Task::none(),
    };
    let map_stem = map_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let gp = game_path.clone();

    Task::perform(
        async move {
            use dispel_core::references::all_map_ini::Map as AllMapI;
            use dispel_core::references::dialogue_paragraph::DialogueParagraph;
            use dispel_core::references::dialogue_script::DialogueScript;

            let all_maps = AllMapI::read_file(&gp.join("AllMap.ini"))
                .map_err(|e| format!("AllMap.ini: {e}"))?;
            let entry = all_maps
                .into_iter()
                .find(|m| m.map_filename.to_lowercase() == map_stem)
                .ok_or_else(|| format!("Map '{map_stem}' not in AllMap.ini"))?;

            let dlg_name = entry
                .dlg_filename
                .as_deref()
                .ok_or_else(|| "No .dlg for this map".to_string())?;
            let pgp_name = entry
                .pgp_filename
                .as_deref()
                .ok_or_else(|| "No .pgp for this map".to_string())?;

            let scripts = DialogueScript::read_file(&gp.join("NpcInGame").join(dlg_name))
                .map_err(|e| format!("{dlg_name}: {e}"))?;
            let paragraphs = DialogueParagraph::read_file(&gp.join("NpcInGame").join(pgp_name))
                .map_err(|e| format!("{pgp_name}: {e}"))?;

            Ok((npc_idx, scripts, paragraphs))
        },
        move |result| Message::map_editor(MapEditorMessage::DialogPreviewLoaded(tab_id, result)),
    )
}

pub fn preview_loaded(
    app: &mut App,
    tab_id: usize,
    result: Result<
        (
            usize,
            Vec<dispel_core::references::dialogue_script::DialogueScript>,
            Vec<dispel_core::references::dialogue_paragraph::DialogueParagraph>,
        ),
        String,
    >,
) -> Task<Message> {
    if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
        match result {
            Ok((npc_idx, scripts, paragraphs)) => {
                // Guard against a stale async result: if the user clicked on
                // a different NPC (or a non-NPC entity) between the time
                // `show_preview` started and now, discard the result instead
                // of showing a preview for the wrong NPC.
                let still_selected = state.view.selected_entity
                    == Some(crate::editors::map_editor::SelectedEntity::Npc(npc_idx));
                if still_selected {
                    state.view.dialog_preview = Some(DialogPreviewState {
                        npc_index: npc_idx,
                        dialog_scripts: scripts,
                        dialog_paragraphs: paragraphs,
                    });
                } else {
                    state.data.notify(
                        gui_widgets::components::toast::Status::Warning,
                        "Dialog",
                        "Preview discarded: NPC selection changed",
                    );
                }
            }
            Err(err) => {
                state.data.notify(
                    gui_widgets::components::toast::Status::Danger,
                    "Error",
                    format!("Dialog preview: {err}"),
                );
            }
        }
    }
    Task::none()
}

pub fn hide_preview(app: &mut App, tab_id: usize) -> Task<Message> {
    if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
        state.view.dialog_preview = None;
    }
    Task::none()
}
