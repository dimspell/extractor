use crate::app::App;
use crate::message::{Message, MessageExt};
use crate::state::RecordingSession;
use hexedit::{HexEditorConfig, HexEditorState};
use iced::widget::{container, text};
use iced::{Element, Fill, Task};
use std::path::PathBuf;

pub fn build_hex_config(
    recording: &Option<RecordingSession>,
    game_path: &Option<PathBuf>,
    state: &HexEditorState,
) -> HexEditorConfig {
    let has_dirty = state.provider.dirty_count() > 0;
    let has_session = recording.is_some();
    let has_game = game_path.is_some();
    let in_game_dir = game_path
        .as_ref()
        .map(|gp| state.path.starts_with(gp))
        .unwrap_or(false);
    let can_save = has_dirty && has_session && has_game && in_game_dir;
    let save_label = match recording {
        Some(s) => format!("Save into `{}`", s.mod_slug),
        None => "Save into recording".to_string(),
    };
    let save_hint = if !has_session {
        "  ·  no recording active".to_string()
    } else if !has_game {
        "  ·  set a game directory".to_string()
    } else if !in_game_dir {
        "  ·  file is outside the game directory".to_string()
    } else if !has_dirty {
        "  ·  no edits to save".to_string()
    } else {
        String::new()
    };
    HexEditorConfig {
        pane_gap: 4,
        on_save: crate::editors::mod_packager::hex_save::build_save_callback(recording, game_path),
        save_label,
        can_save,
        save_hint,
        extra_entries: state.lua_engine.entries(),
        custom_encodings: Vec::new(),
        on_write_mode_changed: None,
    }
}

pub fn handle(msg: hexedit::HexEditorMessage, app: &mut App) -> Task<Message> {
    let tab_id = app
        .state
        .workspace
        .active()
        .map(|t| t.id)
        .unwrap_or(usize::MAX);
    let Some(state) = app.state.editors.hex_editors.get_mut(&tab_id) else {
        return Task::none();
    };
    let config = build_hex_config(&app.state.recording, &app.state.workspace.game_path, state);
    hexedit::update(state, &config, msg).map(Message::hex_editor)
}

pub fn view(app: &App) -> Element<'_, Message> {
    let tab_id = app
        .state
        .workspace
        .active()
        .map(|t| t.id)
        .unwrap_or(usize::MAX);
    match app.state.editors.hex_editors.get(&tab_id) {
        Some(state) => {
            let config =
                build_hex_config(&app.state.recording, &app.state.workspace.game_path, state);
            hexedit::view(state, &config).map(Message::hex_editor)
        }
        None => container(text("Hex editor not loaded").size(14))
            .width(Fill)
            .height(Fill)
            .padding(16)
            .accessible_label("Hex editor")
            .into(),
    }
}
