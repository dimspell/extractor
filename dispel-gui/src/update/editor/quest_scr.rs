// QuestScr editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::quest_scr::QuestScrEditorMessage;
use dispel_core::{Extractor, Quest};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: QuestScrEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        QuestScrEditorMessage::LoadCatalog => {
            if app.state.shared_game_path.is_empty() {
                app.state.quest_scr_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.quest_scr_editor.loading_state = crate::loading_state::LoadingState::Loading;
            app.state.quest_scr_spreadsheet.is_loading = true;
            let path = PathBuf::from(&app.state.shared_game_path)
                .join("ExtraInGame")
                .join("Quest.scr");
            Task::perform(
                async move { Quest::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                move |result: Result<Vec<dispel_core::Quest>, String>| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::QuestScr(
                            QuestScrEditorMessage::CatalogLoaded(result),
                        ),
                    )
                },
            )
        }
        QuestScrEditorMessage::CatalogLoaded(result) => {
            app.state.quest_scr_editor.loading_state =
                crate::loading_state::LoadingState::Loaded(());
            match result {
                Ok(catalog) => {
                    app.state.quest_scr_editor.catalog = Some(catalog.clone());
                    app.state.quest_scr_editor.status_msg =
                        format!("Quest catalog loaded: {} entries", catalog.len());
                    app.state.quest_scr_editor.refresh_quests();
                    app.state.quest_scr_editor.init_pane_state();
                    app.state.quest_scr_spreadsheet.apply_filter(&catalog);
                    app.state.quest_scr_spreadsheet.compute_all_caches(&catalog);
                    app.state.quest_scr_spreadsheet.is_loading = false;
                }
                Err(e) => {
                    app.state.quest_scr_editor.status_msg =
                        format!("Error loading quest catalog: {}", e);
                    app.state.quest_scr_spreadsheet.is_loading = false;
                }
            }
            Task::none()
        }
        QuestScrEditorMessage::Select(index) => {
            app.state.quest_scr_editor.select(index);
            Task::none()
        }
        QuestScrEditorMessage::FieldChanged(index, field, value) => {
            app.state
                .quest_scr_editor
                .update_field(index, &field, value);
            Task::none()
        }
        QuestScrEditorMessage::DescriptionAction(_index, _action) => Task::none(),
        QuestScrEditorMessage::Save => {
            if app.state.shared_game_path.is_empty() {
                app.state.quest_scr_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.quest_scr_editor.loading_state = crate::loading_state::LoadingState::Loading;
            let result = app
                .state
                .quest_scr_editor
                .save_quests(&app.state.shared_game_path);
            app.state.quest_scr_editor.loading_state =
                crate::loading_state::LoadingState::Loaded(());
            match result {
                Ok(_) => {
                    app.state.quest_scr_editor.status_msg = "Quests saved successfully.".into()
                }
                Err(e) => {
                    app.state.quest_scr_editor.status_msg = format!("Error saving quests: {}", e)
                }
            }
            Task::none()
        }
        QuestScrEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages!(
                app,
                quest_scr_spreadsheet,
                quest_scr_editor,
                |index, field, value| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::QuestScr(
                            QuestScrEditorMessage::FieldChanged(index, field, value),
                        ),
                    )
                },
                msg
            );
            Task::none()
        }
        QuestScrEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = app.state.quest_scr_editor.pane_state {
                ps.resize(event.split, event.ratio);
            }
            if let Some(ref mut ps) = app.state.quest_scr_spreadsheet.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }
        QuestScrEditorMessage::PaneClicked(pane) => {
            app.state.quest_scr_editor.pane_focus = Some(pane);
            Task::none()
        }
    }
}
