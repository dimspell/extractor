use crate::app::App;
use crate::editors::event_scr::message::EventScrEditorMessage;
use crate::editors::event_scr::state::{EventScriptEditorState, SectionTab};
use crate::message::Message;
use dispel_core::references::event_scr::{ActionFunction, EventScript, SpriteDefinition, Variable};
use iced::Task;

pub fn handle(message: EventScrEditorMessage, app: &mut App) -> Task<Message> {
    let state: &mut EventScriptEditorState = &mut app.state.event_scr_editor;
    match message {
        EventScrEditorMessage::SectionChanged(tab) => {
            state.active_section = tab;
            Task::none()
        }
        EventScrEditorMessage::VariableAdded => {
            if let Some(ref mut script) = state.script {
                script.variables.push(Variable {
                    name: String::new(),
                    value: String::new(),
                });
                state.modified = true;
            }
            Task::none()
        }
        EventScrEditorMessage::VariableNameChanged(index, name) => {
            if let Some(ref mut script) = state.script {
                if let Some(var) = script.variables.get_mut(index) {
                    var.name = name;
                    state.modified = true;
                }
            }
            Task::none()
        }
        EventScrEditorMessage::VariableValueChanged(index, value) => {
            if let Some(ref mut script) = state.script {
                if let Some(var) = script.variables.get_mut(index) {
                    var.value = value;
                    state.modified = true;
                }
            }
            Task::none()
        }
        EventScrEditorMessage::VariableDeleted(index) => {
            if let Some(ref mut script) = state.script {
                if index < script.variables.len() {
                    script.variables.remove(index);
                    state.modified = true;
                }
            }
            Task::none()
        }
        EventScrEditorMessage::LineAdded(section) => {
            if let Some(ref mut script) = state.script {
                match section {
                    SectionTab::Map => script.map_content.push(String::new()),
                    SectionTab::Chr => script.chr_content.push(String::new()),
                    SectionTab::Npc => script.npc_content.push(String::new()),
                    SectionTab::Wav => script.wav_content.push(String::new()),
                    _ => {}
                }
                state.modified = true;
            }
            Task::none()
        }
        EventScrEditorMessage::LineContentChanged(section, index, content) => {
            if let Some(ref mut script) = state.script {
                match section {
                    SectionTab::Map if index < script.map_content.len() => {
                        script.map_content[index] = content;
                        state.modified = true;
                    }
                    SectionTab::Chr if index < script.chr_content.len() => {
                        script.chr_content[index] = content;
                        state.modified = true;
                    }
                    SectionTab::Npc if index < script.npc_content.len() => {
                        script.npc_content[index] = content;
                        state.modified = true;
                    }
                    SectionTab::Wav if index < script.wav_content.len() => {
                        script.wav_content[index] = content;
                        state.modified = true;
                    }
                    _ => {}
                }
            }
            Task::none()
        }
        EventScrEditorMessage::LineDeleted(section, index) => {
            if let Some(ref mut script) = state.script {
                match section {
                    SectionTab::Map if index < script.map_content.len() => {
                        script.map_content.remove(index);
                        state.modified = true;
                    }
                    SectionTab::Chr if index < script.chr_content.len() => {
                        script.chr_content.remove(index);
                        state.modified = true;
                    }
                    SectionTab::Npc if index < script.npc_content.len() => {
                        script.npc_content.remove(index);
                        state.modified = true;
                    }
                    SectionTab::Wav if index < script.wav_content.len() => {
                        script.wav_content.remove(index);
                        state.modified = true;
                    }
                    _ => {}
                }
            }
            Task::none()
        }
        EventScrEditorMessage::SpriteAdded => {
            if let Some(ref mut script) = state.script {
                script.spr_content.push(SpriteDefinition {
                    sprite_alias: String::new(),
                    sprite_file: String::new(),
                });
                state.modified = true;
            }
            Task::none()
        }
        EventScrEditorMessage::SpriteAliasChanged(index, alias) => {
            if let Some(ref mut script) = state.script {
                if let Some(spr) = script.spr_content.get_mut(index) {
                    spr.sprite_alias = alias;
                    state.modified = true;
                }
            }
            Task::none()
        }
        EventScrEditorMessage::SpriteFileChanged(index, file) => {
            if let Some(ref mut script) = state.script {
                if let Some(spr) = script.spr_content.get_mut(index) {
                    spr.sprite_file = file;
                    state.modified = true;
                }
            }
            Task::none()
        }
        EventScrEditorMessage::SpriteDeleted(index) => {
            if let Some(ref mut script) = state.script {
                if index < script.spr_content.len() {
                    script.spr_content.remove(index);
                    state.modified = true;
                }
            }
            Task::none()
        }
        EventScrEditorMessage::ActionAdded => {
            if let Some(ref mut script) = state.script {
                script.actions.push(ActionFunction {
                    prefix: None,
                    function_name: String::new(),
                    parameters: Vec::new(),
                    raw_content: None,
                });
                state.modified = true;
                state.act_parse_errors = validate_script(script);
            }
            Task::none()
        }
        EventScrEditorMessage::ActionRawAdded => {
            if let Some(ref mut script) = state.script {
                script.actions.push(ActionFunction {
                    prefix: None,
                    function_name: String::new(),
                    parameters: Vec::new(),
                    raw_content: Some(String::new()),
                });
                state.modified = true;
                state.act_parse_errors = validate_script(script);
            }
            Task::none()
        }
        EventScrEditorMessage::ActionRawContentChanged(index, content) => {
            if let Some(ref mut script) = state.script {
                if let Some(act) = script.actions.get_mut(index) {
                    act.raw_content = Some(content);
                    act.prefix = None;
                    act.function_name = String::new();
                    act.parameters = Vec::new();
                    state.modified = true;
                    state.act_parse_errors = validate_script(script);
                }
            }
            Task::none()
        }
        EventScrEditorMessage::ActionPrefixChanged(index, prefix) => {
            if let Some(ref mut script) = state.script {
                if let Some(act) = script.actions.get_mut(index) {
                    act.prefix = if prefix.is_empty() {
                        None
                    } else {
                        Some(prefix)
                    };
                    act.raw_content = None;
                    state.modified = true;
                    state.act_parse_errors = validate_script(script);
                }
            }
            Task::none()
        }
        EventScrEditorMessage::ActionFunctionChanged(index, func_name) => {
            if let Some(ref mut script) = state.script {
                if let Some(act) = script.actions.get_mut(index) {
                    act.function_name = func_name;
                    act.raw_content = None;
                    state.modified = true;
                    state.act_parse_errors = validate_script(script);
                }
            }
            Task::none()
        }
        EventScrEditorMessage::ActionParamsChanged(index, params_str) => {
            if let Some(ref mut script) = state.script {
                if let Some(act) = script.actions.get_mut(index) {
                    act.parameters = params_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    act.raw_content = None;
                    state.modified = true;
                    state.act_parse_errors = validate_script(script);
                }
            }
            Task::none()
        }
        EventScrEditorMessage::ActionDeleted(index) => {
            if let Some(ref mut script) = state.script {
                if index < script.actions.len() {
                    script.actions.remove(index);
                    state.modified = true;
                    state.act_parse_errors = validate_script(script);
                }
            }
            Task::none()
        }
        EventScrEditorMessage::LoadScript(path) => load_from_path(path),
        EventScrEditorMessage::ScriptLoaded(script) => {
            state.script = Some(script);
            state.modified = false;
            state.load_error = None;
            state.save_error = None;
            Task::none()
        }
        EventScrEditorMessage::LoadError(e) => {
            state.script = None;
            state.load_error = Some(e);
            Task::none()
        }
        EventScrEditorMessage::SaveScript => {
            if let Some(ref script) = state.script {
                if let Some(ref path) = state.file_path {
                    return save_to_path(path.clone(), script.clone());
                }
            }
            Task::none()
        }
        EventScrEditorMessage::SaveSuccess => {
            state.modified = false;
            state.save_error = None;
            Task::none()
        }
        EventScrEditorMessage::SaveError(e) => {
            state.save_error = Some(e);
            Task::none()
        }
    }
}

// Helper to load EventScript from path
pub fn load_from_path(path: std::path::PathBuf) -> Task<Message> {
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                <EventScript as dispel_core::Extractor>::read_file(&path)
                    .map(|mut scripts| {
                        if let Some(script) = scripts.pop() {
                            Ok(script)
                        } else {
                            Err(std::io::Error::other("No EventScript found in file"))
                        }
                    })
                    .and_then(|res| res)
            })
            .await
            .unwrap_or_else(|e| Err(std::io::Error::other(e)))
        },
        |result| {
            Message::Editor(crate::message::editor::EditorMessage::EventScr(
                match result {
                    Ok(script) => EventScrEditorMessage::ScriptLoaded(script),
                    Err(e) => EventScrEditorMessage::LoadError(e.to_string()),
                },
            ))
        },
    )
}

fn validate_actions(script: &EventScript) -> Vec<(usize, String)> {
    let mut errors = Vec::new();

    for (idx, action) in script.actions.iter().enumerate() {
        if action.raw_content.is_some() {
            continue;
        }

        if action.function_name.is_empty() && action.prefix.is_none() {
            continue;
        }

        let fn_with_params = if let Some(ref prefix) = action.prefix {
            format!("{}${}", prefix, action.function_name)
        } else {
            action.function_name.clone()
        };

        let params_str = action.parameters.join(",");
        let full_call = format!("{}({})", fn_with_params, params_str);

        let open_parens = full_call.matches('(').count();
        let close_parens = full_call.matches(')').count();

        if open_parens != close_parens {
            if open_parens > close_parens {
                errors.push((
                    idx,
                    format!(
                        "Missing {} closing parenthesis(es)",
                        open_parens - close_parens
                    ),
                ));
            } else {
                errors.push((
                    idx,
                    format!(
                        "Extra {} closing parenthesis(es)",
                        close_parens - open_parens
                    ),
                ));
            }
        }

        if action
            .function_name
            .contains(|c: char| !c.is_alphanumeric() && c != '_')
            && !action.function_name.is_empty()
        {
            errors.push((idx, "Function name contains invalid characters".to_string()));
        }
    }

    errors
}

pub fn validate_script(script: &EventScript) -> Vec<(usize, String)> {
    validate_actions(script)
}

// Helper to save EventScript to path
pub fn save_to_path(path: std::path::PathBuf, script: EventScript) -> Task<Message> {
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                let mut file = std::fs::File::create(&path)?;
                <EventScript as dispel_core::Extractor>::to_writer(&[script], &mut file)
            })
            .await
            .unwrap_or_else(|e| Err(std::io::Error::other(e)))
        },
        |result| {
            Message::Editor(crate::message::editor::EditorMessage::EventScr(
                match result {
                    Ok(()) => EventScrEditorMessage::SaveSuccess,
                    Err(e) => EventScrEditorMessage::SaveError(e.to_string()),
                },
            ))
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use dispel_core::references::event_scr::{ActionFunction, EventScript};

    fn create_test_script() -> EventScript {
        EventScript {
            id: 1,
            header_comments: vec![],
            variables: vec![],
            map_content: vec![],
            chr_content: vec![],
            npc_content: vec![],
            spr_content: vec![],
            wav_content: vec![],
            actions: vec![],
        }
    }

    #[test]
    fn test_validation_valid_action() {
        let mut script = create_test_script();
        script.actions.push(ActionFunction {
            prefix: Some("King".to_string()),
            function_name: "setmappos".to_string(),
            parameters: vec!["10".to_string(), "20".to_string()],
            raw_content: None,
        });
        let errors = validate_script(&script);
        assert!(errors.is_empty(), "Expected no errors for valid action");
    }

    #[test]
    fn test_validation_missing_closing_paren() {
        let mut script = create_test_script();
        script.actions.push(ActionFunction {
            prefix: None,
            function_name: "addquest".to_string(),
            parameters: vec!["1".to_string(), "2".to_string()],
            raw_content: None,
        });
        let errors = validate_script(&script);
        assert!(errors.is_empty(), "Valid function should have no errors");
    }

    #[test]
    fn test_validation_unbalanced_parens() {
        let mut script = create_test_script();
        script.actions.push(ActionFunction {
            prefix: None,
            function_name: "missing_paren".to_string(),
            parameters: vec!["param(".to_string()],
            raw_content: None,
        });
        let errors = validate_script(&script);
        assert!(!errors.is_empty(), "Should detect unbalanced parens");
    }

    #[test]
    fn test_validation_raw_content_no_errors() {
        let mut script = create_test_script();
        script.actions.push(ActionFunction {
            prefix: None,
            function_name: String::new(),
            parameters: vec![],
            raw_content: Some("if(condition)".to_string()),
        });
        let errors = validate_script(&script);
        assert!(errors.is_empty(), "Raw content should not be validated");
    }

    #[test]
    fn test_validation_multiple_actions() {
        let mut script = create_test_script();
        script.actions.push(ActionFunction {
            prefix: None,
            function_name: "action1".to_string(),
            parameters: vec!["1".to_string()],
            raw_content: None,
        });
        script.actions.push(ActionFunction {
            prefix: Some("NPC".to_string()),
            function_name: "dialog".to_string(),
            parameters: vec!["hello".to_string()],
            raw_content: None,
        });
        let errors = validate_script(&script);
        assert!(
            errors.is_empty(),
            "Multiple valid actions should have no errors"
        );
    }

    #[test]
    fn test_validation_empty_function_name() {
        let mut script = create_test_script();
        script.actions.push(ActionFunction {
            prefix: None,
            function_name: String::new(),
            parameters: vec![],
            raw_content: None,
        });
        let errors = validate_script(&script);
        assert!(
            errors.is_empty(),
            "Empty function should not produce errors"
        );
    }
}
