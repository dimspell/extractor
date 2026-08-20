use crate::app::App;
use crate::editors::map_editor::message::ConversationLoadResult;
use crate::editors::map_editor::state::ConversationState;
use crate::editors::map_editor::{ChoiceOption, ConversationLine, MapEditorMessage};
use crate::message::{Message, MessageExt};
use dispel_core::references::dialogue_paragraph::DialogueParagraph;
use dispel_core::references::dialogue_script::DialogueScript;
use dispel_core::references::enums::{DialogOwner, DialogType};
use iced::Task;

/// Start an interactive conversation with an NPC.
pub fn start(app: &mut App, tab_id: usize, npc_idx: usize) -> Task<Message> {
    let state = match app.state.editors.map_editors.get(&tab_id) {
        Some(s) => s,
        None => return Task::none(),
    };

    let npc = match state.data.npcs.get(npc_idx) {
        Some(n) => n,
        None => return Task::none(),
    };

    let entry_dialog_id = npc.dialog_id;
    if entry_dialog_id == 0 {
        if let Some(editor) = app.state.editors.map_editors.get_mut(&tab_id) {
            editor.data.status_msg = Some("NPC has no dialog".into());
        }
        return Task::none();
    }

    // We need the scripts/paragraphs. Use the ones from dialog_preview if
    // available; otherwise load them asynchronously.
    if let Some(preview) = &state.view.dialog_preview {
        let mut conv = ConversationState {
            npc_index: npc_idx,
            npc_name: npc.name.clone(),
            scripts: preview.dialog_scripts.clone(),
            paragraphs: preview.dialog_paragraphs.clone(),
            entry_dialog_id,
            current_node_id: Some(entry_dialog_id),
            history: Vec::new(),
            executed_events: std::collections::HashSet::new(),
            choices: Vec::new(),
            waiting_for_advance: false,
            finished: false,
        };
        resolve_and_display(&mut conv);
        if let Some(editor) = app.state.editors.map_editors.get_mut(&tab_id) {
            editor.view.conversation = Some(conv);
        }
        Task::none()
    } else {
        // No preview data — need to load DLG/PGP first
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
        let npc_name = npc.name.clone();

        Task::perform(
            async move {
                use dispel_core::references::all_map_ini::Map as AllMapI;
                use dispel_core::references::extractor::Extractor;

                let all_maps = AllMapI::read_file(&game_path.join("AllMap.ini"))
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

                let scripts =
                    DialogueScript::read_file(&game_path.join("NpcInGame").join(dlg_name))
                        .map_err(|e| format!("{dlg_name}: {e}"))?;
                let paragraphs =
                    DialogueParagraph::read_file(&game_path.join("NpcInGame").join(pgp_name))
                        .map_err(|e| format!("{pgp_name}: {e}"))?;

                Ok::<_, String>((npc_idx, npc_name, entry_dialog_id, scripts, paragraphs))
            },
            move |result| Message::map_editor(MapEditorMessage::ConversationLoaded(tab_id, result)),
        )
    }
}

/// Handle async conversation data loaded.
pub fn loaded(app: &mut App, tab_id: usize, result: ConversationLoadResult) -> Task<Message> {
    if let Some(editor) = app.state.editors.map_editors.get_mut(&tab_id) {
        match result {
            Ok((npc_idx, npc_name, entry_dialog_id, scripts, paragraphs)) => {
                let mut conv = ConversationState {
                    npc_index: npc_idx,
                    npc_name,
                    scripts,
                    paragraphs,
                    entry_dialog_id,
                    current_node_id: Some(entry_dialog_id),
                    history: Vec::new(),
                    executed_events: std::collections::HashSet::new(),
                    choices: Vec::new(),
                    waiting_for_advance: false,
                    finished: false,
                };
                resolve_and_display(&mut conv);
                editor.view.conversation = Some(conv);
            }
            Err(err) => {
                editor.data.status_msg = Some(format!("Conversation: {err}"));
            }
        }
    }
    Task::none()
}

/// Advance the conversation (click to continue).
pub fn advance(app: &mut App, tab_id: usize) -> Task<Message> {
    if let Some(editor) = app.state.editors.map_editors.get_mut(&tab_id)
        && let Some(ref mut conv) = editor.view.conversation
    {
        if conv.finished || !conv.waiting_for_advance {
            return Task::none();
        }
        conv.waiting_for_advance = false;
        // Follow next_dialog_id1 (the linear next)
        if let Some(current_id) = conv.current_node_id
            && let Some(script) = conv.scripts.iter().find(|s| s.id == current_id)
        {
            // Fire triggered event and notify
            if let Some(evt) = script.triggered_event_id.filter(|&id| id != 0) {
                conv.executed_events.insert(evt);
                conv.history.push(ConversationLine {
                    speaker: "Event".into(),
                    text: format!("Triggered event {evt}"),
                    is_choice: false,
                    locked: false,
                    locked_event_id: None,
                    is_system: true,
                });
            }
            // Follow next_dialog_id1 only — next_dialog_to_check is for
            // the event gate chain, not the main dialog flow.
            let next_id = script.next_dialog_id1.filter(|&id| id != 0);
            if let Some(nid) = next_id {
                conv.current_node_id = Some(nid);
                resolve_and_display(conv);
            } else {
                conv.current_node_id = None;
                conv.finished = true;
            }
        }
    }
    Task::none()
}

/// Select a choice option.
pub fn select_choice(app: &mut App, tab_id: usize, choice_index: usize) -> Task<Message> {
    if let Some(editor) = app.state.editors.map_editors.get_mut(&tab_id)
        && let Some(ref mut conv) = editor.view.conversation
    {
        if conv.finished || conv.choices.is_empty() {
            return Task::none();
        }
        if choice_index >= conv.choices.len() {
            return Task::none();
        }

        let choice = &conv.choices[choice_index];
        let target = choice.target_node_id;

        // Fire the current node's triggered event
        if let Some(current_id) = conv.current_node_id
            && let Some(script) = conv.scripts.iter().find(|s| s.id == current_id)
            && let Some(evt) = script.triggered_event_id.filter(|&id| id != 0)
        {
            conv.executed_events.insert(evt);
            conv.history.push(ConversationLine {
                speaker: "Event".into(),
                text: format!("Triggered event {evt}"),
                is_choice: false,
                locked: false,
                locked_event_id: None,
                is_system: true,
            });
        }

        conv.choices.clear();
        conv.waiting_for_advance = false;

        if target != 0 {
            conv.current_node_id = Some(target);
            resolve_and_display(conv);
        } else {
            conv.current_node_id = None;
            conv.finished = true;
        }
    }
    Task::none()
}

/// Reset the conversation to the beginning.
pub fn reset(app: &mut App, tab_id: usize) -> Task<Message> {
    if let Some(editor) = app.state.editors.map_editors.get_mut(&tab_id)
        && let Some(ref mut conv) = editor.view.conversation
    {
        conv.current_node_id = Some(conv.entry_dialog_id);
        conv.history.clear();
        conv.executed_events.clear();
        conv.choices.clear();
        conv.waiting_for_advance = false;
        conv.finished = false;
        resolve_and_display(conv);
    }
    Task::none()
}

/// Close the conversation display.
pub fn close(app: &mut App, tab_id: usize) -> Task<Message> {
    if let Some(editor) = app.state.editors.map_editors.get_mut(&tab_id) {
        editor.view.conversation = None;
    }
    Task::none()
}

// ── Core state machine ────────────────────────────────────────────────────────

/// Resolve the current node: walk event gate chains showing locked entries,
/// then display the first unlocked node.
fn resolve_and_display(conv: &mut ConversationState) {
    let Some(start_id) = conv.current_node_id else {
        conv.finished = true;
        return;
    };

    const MAX_CHAIN: usize = 50;
    let mut current = start_id;

    for _ in 0..MAX_CHAIN {
        let script = match conv.scripts.iter().find(|s| s.id == current) {
            Some(s) => s,
            None => {
                conv.finished = true;
                return;
            }
        };

        // Check event gate
        if let Some(req_event) = script.required_event_id.filter(|&id| id != 0)
            && !conv.executed_events.contains(&req_event)
        {
            // Gate not satisfied — show as locked, then follow chain
            let text = load_pgp_text(conv, script);
            let speaker = get_speaker(conv, script);

            conv.history.push(ConversationLine {
                speaker,
                text,
                is_choice: false,
                locked: true,
                locked_event_id: Some(req_event),
                is_system: false,
            });

            if let Some(next) = script.next_dialog_to_check.filter(|&id| id != 0) {
                current = next;
                continue;
            }
            conv.finished = true;
            return;
        }

        // Gate passed (or no gate) — display normally
        conv.current_node_id = Some(current);
        let text = load_pgp_text(conv, script);
        let speaker = get_speaker(conv, script);

        match script.dialog_type {
            Some(DialogType::Choice) => {
                conv.history.push(ConversationLine {
                    speaker,
                    text,
                    is_choice: false,
                    locked: false,
                    locked_event_id: None,
                    is_system: false,
                });

                conv.choices.clear();
                let labels = ["A", "B", "C"];
                for (i, opt_id) in [
                    script.next_dialog_id1,
                    script.next_dialog_id2,
                    script.next_dialog_id3,
                ]
                .iter()
                .enumerate()
                {
                    if let Some(nid) = opt_id.filter(|&id| id != 0) {
                        let choice_text = resolve_event_chain(conv, nid)
                            .and_then(|rid| conv.scripts.iter().find(|s| s.id == rid))
                            .and_then(|s| {
                                s.dialog_id.and_then(|did| {
                                    conv.paragraphs.iter().find(|p| p.id == did).map(|p| {
                                        let t = p.text.replace('$', " ");
                                        if t.len() > 40 {
                                            format!("{}…", &t[..37])
                                        } else {
                                            t
                                        }
                                    })
                                })
                            })
                            .unwrap_or_else(|| format!("Option {}", labels[i]));

                        conv.choices.push(ChoiceOption {
                            label: format!("{}. {}", labels[i], choice_text),
                            target_node_id: nid,
                            triggered_event_id: 0,
                        });
                    }
                }
                conv.waiting_for_advance = false;
            }
            _ => {
                conv.history.push(ConversationLine {
                    speaker,
                    text,
                    is_choice: false,
                    locked: false,
                    locked_event_id: None,
                    is_system: false,
                });
                conv.choices.clear();
                conv.waiting_for_advance = true;
            }
        }
        return;
    }

    conv.finished = true;
}

fn load_pgp_text(conv: &ConversationState, script: &DialogueScript) -> String {
    script
        .dialog_id
        .and_then(|did| conv.paragraphs.iter().find(|p| p.id == did))
        .map(|p| p.text.replace('$', "\n"))
        .unwrap_or_else(|| "[text not found]".to_string())
}

fn get_speaker(conv: &ConversationState, script: &DialogueScript) -> String {
    match script.dialog_owner {
        Some(DialogOwner::Npc) => conv.npc_name.clone(),
        Some(DialogOwner::Player) => "Player".to_string(),
        None => "?".to_string(),
    }
}

/// Walk the `next_dialog_to_check` chain, skipping nodes whose
/// `required_event_id` is not in `executed_events`.
fn resolve_event_chain(conv: &ConversationState, start_id: i32) -> Option<i32> {
    const MAX_CHAIN: usize = 50;
    let mut current = start_id;

    for _ in 0..MAX_CHAIN {
        let script = conv.scripts.iter().find(|s| s.id == current)?;

        // Check event gate
        if let Some(req_event) = script.required_event_id.filter(|&id| id != 0)
            && !conv.executed_events.contains(&req_event)
        {
            // Gate not satisfied — follow the chain
            if let Some(next) = script.next_dialog_to_check.filter(|&id| id != 0) {
                current = next;
                continue;
            }
            return None;
        }

        // Gate passed (or no gate)
        return Some(current);
    }

    None
}
