use super::Command;
use dispel_core::references::dialogue_paragraph::read_dialogue_paragraphs;
use dispel_core::references::dialogue_script::{read_dialogs, DialogueScript};
use dispel_core::references::npc_ref::read_npc_ref;
use dispel_core::DialogType;
use rusqlite::{Connection, Result as SqlResult};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Clone)]
struct NpcInfo {
    name: String,
    description: String,
}

/// Information about an event from the database
#[derive(Debug, Clone)]
struct EventInfo {
    #[expect(dead_code)]
    event_id: i32,
    event_filename: Option<String>,
    actions: Vec<EventAction>,
}

/// A single event action that can execute dialog functions
#[derive(Debug, Clone)]
struct EventAction {
    action_order: i32,
    function_name: String,
    parameters: Option<String>,
}

/// Dialog function information extracted from event actions
#[derive(Debug, Clone)]
struct DialogFunctionCall {
    event_id: i32,
    action_order: i32,
    function_name: String,
    parameters: String,
    /// Which dialog IDs this action references
    dialog_ids: Vec<i32>,
}

pub struct DialogCommand {
    pub dlg_path: String,
    pub pgp_path: Option<String>,
    pub npc_ref_path: Option<String>,
    pub database_path: Option<String>,
}

impl Command for DialogCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        let dlg_path = Path::new(&self.dlg_path);
        let dialogs =
            read_dialogs(dlg_path).map_err(|e| format!("ERROR: could not read DLG file: {e}"))?;

        let texts: HashMap<i32, String> = if let Some(pgp_path) = &self.pgp_path {
            let pgp_path = Path::new(pgp_path);
            let dialogue_paragraphs = read_dialogue_paragraphs(pgp_path)
                .map_err(|e| format!("ERROR: could not read PGP file: {e}"))?;
            dialogue_paragraphs
                .into_iter()
                .map(|t| (t.id, t.text))
                .collect()
        } else {
            HashMap::new()
        };

        let npcs: HashMap<i32, NpcInfo> = if let Some(npc_ref_path) = &self.npc_ref_path {
            let npc_ref_path = Path::new(npc_ref_path);
            let npc_list = read_npc_ref(npc_ref_path)
                .map_err(|e| format!("ERROR: could not read NPC ref file: {e}"))?;
            npc_list
                .into_iter()
                .filter(|n| n.dialog_id != 0)
                .map(|n| {
                    (
                        n.dialog_id,
                        NpcInfo {
                            name: n.name.trim().to_string(),
                            description: n.description.trim().to_string(),
                        },
                    )
                })
                .collect()
        } else {
            HashMap::new()
        };

        // Load event information from database if provided
        let event_info: HashMap<i32, EventInfo> = if let Some(db_path) = &self.database_path {
            load_event_information(Path::new(db_path)).unwrap_or_else(|e| {
                eprintln!(
                    "WARNING: Could not load event information from database: {}",
                    e
                );
                HashMap::new()
            })
        } else {
            HashMap::new()
        };

        // Extract dialog function calls from events
        let mut dialog_functions: Vec<DialogFunctionCall> = if !event_info.is_empty() {
            extract_dialog_functions(&event_info)
        } else {
            Vec::new()
        };

        // Filter dialog functions to only show those relevant to the current DLG file
        // Collect all dialog IDs from the current file
        let current_dialog_ids: HashSet<i32> = dialogs.iter().map(|d| d.id).collect();

        // For each dialog function call, check if any of its dialog IDs match the current file
        if !dialog_functions.is_empty() {
            dialog_functions.retain(|func| {
                // Keep if any of the dialog IDs from the function parameters match our current dialogs
                func.dialog_ids
                    .iter()
                    .any(|id| current_dialog_ids.contains(id))
            });
        }

        print_dialog_event_graph(&dialogs, &texts, &npcs, &event_info, &dialog_functions);
        Ok(())
    }
}

/// Load all event information from the SQLite database
fn load_event_information(db_path: &Path) -> SqlResult<HashMap<i32, EventInfo>> {
    let conn = Connection::open(db_path)?;

    // Get all events
    let mut stmt = conn.prepare("SELECT event_id, event_filename FROM events")?;

    let event_rows = stmt.query_map([], |row| {
        Ok((row.get::<_, i32>(0)?, row.get::<_, Option<String>>(1)?))
    })?;

    let mut events_map: HashMap<i32, EventInfo> = HashMap::new();

    for event_row in event_rows {
        let (event_id, event_filename) = event_row?;
        let mut event_info = EventInfo {
            event_id,
            event_filename: event_filename.clone(),
            actions: Vec::new(),
        };

        // Get actions for this event
        let mut action_stmt = conn.prepare(
            "SELECT action_order, function_name, parameters
             FROM event_actions
             WHERE event_id = ?1
             ORDER BY action_order",
        )?;

        let action_rows = action_stmt.query_map([event_id], |row| {
            Ok(EventAction {
                action_order: row.get(0)?,
                function_name: row.get(1)?,
                parameters: row.get(2)?,
            })
        })?;

        for action in action_rows {
            event_info.actions.push(action?);
        }

        events_map.insert(event_id, event_info);
    }

    Ok(events_map)
}

/// Extract dialog-related function calls from event actions
/// Based on EVENT_SCRIPT_FUNCTIONS.md, only these 3 functions are dialog-related:
/// - dialogtonpc(npc_id, dialog_id) - Show dialogue to specific NPC
/// - dialog(dialog_id, sprite_file, speaker_name, animation, face_id) - General dialogue
/// - dialogtoparty(dialog_id, speaker_idx) - Dialogue to party
fn extract_dialog_functions(event_info: &HashMap<i32, EventInfo>) -> Vec<DialogFunctionCall> {
    let mut functions = Vec::new();

    // Only these 3 functions are actually dialog-related
    let dialog_functions: std::collections::HashSet<&str> =
        ["dialogtonpc", "dialog", "dialogtoparty"]
            .iter()
            .cloned()
            .collect();

    for (event_id, event) in event_info {
        for action in &event.actions {
            let func_name = action.function_name.to_lowercase();

            // Only match exact dialog function names
            if dialog_functions.contains(func_name.as_str()) {
                // Parse parameters to extract dialog IDs
                let dialog_ids =
                    parse_dialog_ids_from_parameters(&action.function_name, &action.parameters);

                functions.push(DialogFunctionCall {
                    event_id: *event_id,
                    action_order: action.action_order,
                    function_name: action.function_name.clone(),
                    parameters: action.parameters.clone().unwrap_or_default(),
                    dialog_ids,
                });
            }
        }
    }

    functions
}

/// Parse dialog IDs from function parameters
/// Parameters by function type:
/// - dialogtonpc(npc_id, dialog_id) - we want the second value (dialog_id)
/// - dialog(dialog_id, sprite_file, speaker_name, animation, face_id) - we want the first value (dialog_id)
/// - dialogtoparty(dialog_id, speaker_idx) - we want the first value (dialog_id)
fn parse_dialog_ids_from_parameters(function_name: &str, params: &Option<String>) -> Vec<i32> {
    let mut ids = Vec::new();

    if let Some(params_str) = params {
        let parts: Vec<&str> = params_str.split(',').collect();

        match function_name {
            "dialogtonpc" => {
                // dialogtonpc(npc_id, dialog_id) - get the second parameter
                if parts.len() >= 2 {
                    let dialog_id_param = parts[1].trim();
                    if dialog_id_param
                        .chars()
                        .all(|c| c.is_ascii_digit() || c == '-')
                    {
                        if let Ok(id) = dialog_id_param.parse::<i32>() {
                            ids.push(id);
                        }
                    }
                }
            }
            "dialog" => {
                // dialog(dialog_id, sprite_file, speaker_name, animation, face_id) - get the first parameter
                if !parts.is_empty() {
                    let dialog_id_param = parts[0].trim();
                    if dialog_id_param
                        .chars()
                        .all(|c| c.is_ascii_digit() || c == '-')
                    {
                        if let Ok(id) = dialog_id_param.parse::<i32>() {
                            ids.push(id);
                        }
                    }
                }
            }
            "dialogtoparty" => {
                // dialogtoparty(dialog_id, speaker_idx) - get the first parameter
                if !parts.is_empty() {
                    let dialog_id_param = parts[0].trim();
                    if dialog_id_param
                        .chars()
                        .all(|c| c.is_ascii_digit() || c == '-')
                    {
                        if let Ok(id) = dialog_id_param.parse::<i32>() {
                            ids.push(id);
                        }
                    }
                }
            }
            _ => {
                // For any other function (shouldn't happen as we filter by dialog functions)
                // Try to get the first numeric parameter
                for part in parts {
                    let trimmed = part.trim();
                    if trimmed.chars().all(|c| c.is_ascii_digit() || c == '-') {
                        if let Ok(id) = trimmed.parse::<i32>() {
                            ids.push(id);
                        }
                    }
                }
            }
        }
    }

    ids
}

fn print_dialog_event_graph(
    dialogs: &[DialogueScript],
    texts: &HashMap<i32, String>,
    npcs: &HashMap<i32, NpcInfo>,
    event_info: &HashMap<i32, EventInfo>,
    dialog_functions: &[DialogFunctionCall],
) {
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                           DIALOG FLOW                                         ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();

    let dialog_map: HashMap<i32, &DialogueScript> = dialogs.iter().map(|d| (d.id, d)).collect();
    let all_dialog_ids: HashSet<i32> = dialogs.iter().map(|d| d.id).collect();

    // Collect all event connections
    let mut event_triggered_by: HashMap<i32, Vec<i32>> = HashMap::new(); // event_id -> dialog_ids
    let mut event_required_by: HashMap<i32, Vec<i32>> = HashMap::new(); // event_id -> dialog_ids

    for dialog in dialogs {
        if dialog.id == 0 {
            continue;
        }

        if let Some(req_event) = dialog.required_event_id {
            if req_event != 0 {
                event_required_by
                    .entry(req_event)
                    .or_default()
                    .push(dialog.id);
            }
        }

        if let Some(trig_event) = dialog.triggered_event_id {
            if trig_event != 0 {
                event_triggered_by
                    .entry(trig_event)
                    .or_default()
                    .push(dialog.id);
            }
        }
    }

    // Build NPC to dialog mapping
    let mut npc_to_dialog: HashMap<i32, Vec<i32>> = HashMap::new();
    for &dialog_id in npcs.keys() {
        // In the current structure, npc_ref has dialog_id which is the starting dialog
        npc_to_dialog.entry(dialog_id).or_default().push(dialog_id);
    }

    let mut printed = HashMap::new();
    let entry_points = find_entry_points(&dialog_map, &all_dialog_ids);

    // Print event summary header
    println!("═════════════════════════════════════════════════════════════════════════════");
    println!("EVENT CONNECTIONS SUMMARY");
    println!("═════════════════════════════════════════════════════════════════════════════");

    // Print database-derived dialog function calls if available
    if !dialog_functions.is_empty() {
        println!("\n📊 DATABASE-DERIVED DIALOG FUNCTION CALLS:");
        println!("   (From event scripts in database)");

        // Group by function name
        let mut functions_by_type: HashMap<String, Vec<&DialogFunctionCall>> = HashMap::new();
        for func in dialog_functions {
            functions_by_type
                .entry(func.function_name.clone())
                .or_default()
                .push(func);
        }

        let mut func_names: Vec<&String> = functions_by_type.keys().collect();
        func_names.sort();

        for func_name in func_names {
            let funcs = &functions_by_type[func_name];
            println!("\n   Function: {}", func_name);

            // Group by event
            let mut funcs_by_event: HashMap<i32, Vec<&DialogFunctionCall>> = HashMap::new();
            for f in funcs {
                funcs_by_event.entry(f.event_id).or_default().push(f);
            }

            let mut event_ids: Vec<i32> = funcs_by_event.keys().cloned().collect();
            event_ids.sort();

            for event_id in event_ids {
                let event_funcs = &funcs_by_event[&event_id];
                let event_filename = event_info
                    .get(&event_id)
                    .and_then(|e| e.event_filename.as_deref())
                    .unwrap_or("unknown");

                for f in event_funcs {
                    let dialog_refs = if !f.dialog_ids.is_empty() {
                        format!(
                            " → DLGs: {}",
                            f.dialog_ids
                                .iter()
                                .map(|d| d.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    } else {
                        String::new()
                    };
                    println!(
                        "      Event {} ({})[action #{}] {}{}",
                        event_id, event_filename, f.action_order, f.parameters, dialog_refs
                    );
                }
            }
        }
        println!();
    }

    // Print events that trigger dialogs (required_event_id)
    if !event_required_by.is_empty() {
        println!("\n  Events that UNLOCK Dialogs (dialogs require these events):");
        let mut event_ids: Vec<i32> = event_required_by.keys().cloned().collect();
        event_ids.sort();
        for event_id in event_ids {
            let dialogs = &event_required_by[&event_id];
            println!(
                "    Event {} → Dialogs: {}",
                event_id,
                dialogs
                    .iter()
                    .map(|d| d.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }

    // Print events triggered by dialogs
    if !event_triggered_by.is_empty() {
        println!("\n  Events TRIGGERED by Dialogs:");
        let mut event_ids: Vec<i32> = event_triggered_by.keys().cloned().collect();
        event_ids.sort();
        for event_id in event_ids {
            let dialogs = &event_triggered_by[&event_id];
            println!(
                "    Event {} ← Dialogs: {}",
                event_id,
                dialogs
                    .iter()
                    .map(|d| d.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }

    if event_required_by.is_empty() && event_triggered_by.is_empty() {
        println!("  No event connections found in this dialog file.");
    }

    println!();
    println!("═════════════════════════════════════════════════════════════════════════════");
    println!("CONVERSATION FLOW (with event connections)");
    println!("═════════════════════════════════════════════════════════════════════════════");
    println!();

    // Print conversations with event markers
    for entry_id in &entry_points {
        if printed.contains_key(entry_id) {
            continue;
        }

        let npc_info = npcs.get(entry_id);
        let npc_label = if let Some(npc) = npc_info {
            if !npc.name.is_empty() {
                format!("{} ({})", npc.name.trim(), npc.description.trim())
            } else {
                npc.description.trim().to_string()
            }
        } else {
            String::new()
        };

        // Check if this entry point has event connections
        let has_event_connection = dialog_map
            .get(entry_id)
            .is_some_and(|d| d.required_event_id != Some(0) || d.triggered_event_id != Some(0));

        let event_marker = if has_event_connection { " ⚡" } else { "" };

        println!("┌──────────────────────────────────────────────────────────────────────────────");
        if npc_label.is_empty() {
            println!("│ CONVERSATION {} (Entry Point){}", entry_id, event_marker);
        } else {
            println!(
                "│ CONVERSATION {} (Entry Point) - {}{}",
                entry_id, npc_label, event_marker
            );
        }
        println!("└──────────────────────────────────────────────────────────────────────────────");
        print_node_recursive(
            &dialog_map,
            texts,
            npcs,
            event_info,
            *entry_id,
            0,
            &mut printed,
        );
        println!();
    }

    let unprinted: Vec<_> = dialogs
        .iter()
        .filter(|d| d.id != 0 && !printed.contains_key(&d.id))
        .collect();

    if !unprinted.is_empty() {
        println!("┌──────────────────────────────────────────────────────────────────────────────");
        println!("│ ORPHANED DIALOGS (not reachable from entry points)");
        println!("└──────────────────────────────────────────────────────────────────────────────");
        for dialog in unprinted {
            let has_events =
                dialog.required_event_id != Some(0) || dialog.triggered_event_id != Some(0);
            let event_marker = if has_events { " ⚡" } else { "" };
            let req_event = dialog.required_event_id.map_or(0, |x| x);
            let trig_event = dialog.triggered_event_id.map_or(0, |x| x);
            let req_marker = if req_event != 0 {
                format!("[Req: E{}]", req_event)
            } else {
                String::new()
            };
            let trig_marker = if trig_event != 0 {
                format!("[Trig: E{}]", trig_event)
            } else {
                String::new()
            };
            let markers = if req_marker.is_empty() && trig_marker.is_empty() {
                String::new()
            } else if req_marker.is_empty() {
                trig_marker
            } else if trig_marker.is_empty() {
                req_marker
            } else {
                format!("{} {}", req_marker, trig_marker)
            };

            println!(
                "└─ DLG {} {} (speaker: {:?}, type: {:?}){}",
                dialog.id, markers, dialog.dialog_owner, dialog.dialog_type, event_marker
            );
        }
        println!();
    }

    // Print detailed event connection analysis at the end
    if !event_required_by.is_empty() || !event_triggered_by.is_empty() {
        println!("═════════════════════════════════════════════════════════════════════════════");
        println!("EVENT CONNECTION DETAILS");
        println!("═════════════════════════════════════════════════════════════════════════════");

        // Events that unlock dialogs
        if !event_required_by.is_empty() {
            println!("\n🔓 Events that UNLOCK Dialogs:");
            let mut event_ids: Vec<i32> = event_required_by.keys().cloned().collect();
            event_ids.sort();
            for event_id in event_ids {
                let dialogs = &event_required_by[&event_id];
                println!(
                    "   Event {} → Unlocks {} dialog(s): {}",
                    event_id,
                    dialogs.len(),
                    dialogs
                        .iter()
                        .map(|d| format!("DLG{}", d))
                        .collect::<Vec<_>>()
                        .join(", ")
                );

                // Show which NPCs or dialogs lead to these gated dialogs
                for &dialog_id in dialogs {
                    if let Some(_dialog) = dialog_map.get(&dialog_id) {
                        // Find who references this dialog
                        let referenced_by: Vec<i32> = dialog_map
                            .values()
                            .filter(|d| {
                                d.id != dialog_id
                                    && (d.next_dialog_id1 == Some(dialog_id)
                                        || d.next_dialog_id2 == Some(dialog_id)
                                        || d.next_dialog_id3 == Some(dialog_id))
                            })
                            .map(|d| d.id)
                            .collect();

                        if !referenced_by.is_empty() {
                            println!(
                                "      DLG{} is referenced by: {}",
                                dialog_id,
                                referenced_by
                                    .iter()
                                    .map(|r| format!("DLG{}", r))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            );
                        }
                    }
                }
            }
        }

        // Events triggered by dialogs
        if !event_triggered_by.is_empty() {
            println!("\n🎯 Events TRIGGERED by Dialogs:");
            let mut event_ids: Vec<i32> = event_triggered_by.keys().cloned().collect();
            event_ids.sort();
            for event_id in event_ids {
                let dialogs = &event_triggered_by[&event_id];
                println!(
                    "   Event {} ← Triggered by {} dialog(s): {}",
                    event_id,
                    dialogs.len(),
                    dialogs
                        .iter()
                        .map(|d| format!("DLG{}", d))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }

        // Bidirectional connections (dialog requires AND triggers events)
        let bidirectional: Vec<i32> = dialogs
            .iter()
            .filter(|d| d.required_event_id != Some(0) && d.triggered_event_id != Some(0))
            .map(|d| d.id)
            .collect();

        if !bidirectional.is_empty() {
            println!("\n🔄 Bidirectional Event-Dialog Connections:");
            for &dialog_id in &bidirectional {
                if let Some(dialog) = dialog_map.get(&dialog_id) {
                    println!(
                        "   DLG{} requires E{} and triggers E{}",
                        dialog_id,
                        dialog.required_event_id.map_or(0, |x| x),
                        dialog.triggered_event_id.map_or(0, |x| x)
                    );
                }
            }
        }

        println!();
    }

    if printed.is_empty() {
        println!("No dialogs found. This file may not contain any conversation entries.");
    }
}

fn is_entry_point(dialog_map: &HashMap<i32, &DialogueScript>, id: i32) -> bool {
    for dialog in dialog_map.values() {
        if dialog.next_dialog_id1 == Some(id)
            || dialog.next_dialog_id2 == Some(id)
            || dialog.next_dialog_id3 == Some(id)
        {
            return false;
        }
    }
    true
}

fn print_node_recursive(
    dialog_map: &HashMap<i32, &DialogueScript>,
    texts: &HashMap<i32, String>,
    npcs: &HashMap<i32, NpcInfo>,
    event_info: &HashMap<i32, EventInfo>,
    id: i32,
    depth: usize,
    printed: &mut HashMap<i32, bool>,
) {
    if id == 0 {
        return;
    }

    if printed.contains_key(&id) {
        println!("{}└─ [→ {} (loopback)]", "   ".repeat(depth), id);
        return;
    }
    printed.insert(id, true);

    let Some(dialog) = dialog_map.get(&id) else {
        println!("{}└─ [DLG {}: not found]", "   ".repeat(depth), id);
        return;
    };

    let dialog_type = dialog.dialog_type.unwrap_or(DialogType::Normal);
    let owner = dialog
        .dialog_owner
        .map(|o| match o {
            dispel_core::DialogOwner::Player => "Player",
            dispel_core::DialogOwner::Npc => "NPC",
        })
        .unwrap_or("Unknown");

    let text = dialog
        .dialog_id
        .and_then(|id| texts.get(&id))
        .map(|s| s.as_str())
        .unwrap_or("");

    let type_str = match dialog_type {
        DialogType::Normal => "[NORMAL]",
        DialogType::Choice => "[CHOICE]",
    };

    // Build event markers
    let req_event = dialog.required_event_id.map_or(0, |x| x);
    let trig_event = dialog.triggered_event_id.map_or(0, |x| x);

    let mut event_markers = Vec::new();
    if req_event != 0 {
        event_markers.push(format!("🔒 E{}", req_event));
    }
    if trig_event != 0 {
        event_markers.push(format!("🎯 E{}", trig_event));
    }
    let event_display = if !event_markers.is_empty() {
        format!(" [{}]", event_markers.join(", "))
    } else {
        String::new()
    };

    println!(
        "{}┌─ DLG {} {} (speaker: {}){}",
        "   ".repeat(depth),
        id,
        type_str,
        owner,
        event_display
    );

    // Show NPC info if this dialog is an NPC entry point
    if depth == 0 {
        if let Some(npc) = npcs.get(&id) {
            println!(
                "{}│   NPC: {} - {}",
                "   ".repeat(depth),
                npc.name,
                npc.description
            );
        }
    }

    if !text.is_empty() {
        let lines: Vec<&str> = text.lines().collect();
        // Limit to 5 lines to avoid overwhelming output
        let max_lines = 5;
        let lines_to_show = lines.len().min(max_lines);
        for line in &lines[..lines_to_show] {
            println!("{}│   \"{}\"", "   ".repeat(depth), line);
        }
        if lines.len() > max_lines {
            println!(
                "{}│   ... ({} more lines)",
                "   ".repeat(depth),
                lines.len() - max_lines
            );
        }
    }

    // Show event connections from the DLG file itself
    if req_event != 0 {
        println!(
            "{}│   🔒 Requires: Event {}",
            "   ".repeat(depth),
            req_event
        );
        // Show event filename from database if available
        if let Some(event) = event_info.get(&req_event) {
            if let Some(ref filename) = event.event_filename {
                println!("{}│      📁 File: {}", "   ".repeat(depth), filename);
            }
        }
    }
    if trig_event != 0 {
        println!(
            "{}│   🎯 Triggers: Event {}",
            "   ".repeat(depth),
            trig_event
        );
        // Show event filename from database if available
        if let Some(event) = event_info.get(&trig_event) {
            if let Some(ref filename) = event.event_filename {
                println!("{}│      📁 File: {}", "   ".repeat(depth), filename);
            }
        }
    }

    match dialog_type {
        DialogType::Normal => {
            if let Some(next) = dialog.next_dialog_id1 {
                if next != 0 {
                    println!("{}│", "   ".repeat(depth));
                    print_node_recursive(dialog_map, texts, npcs, event_info, next, depth, printed);
                }
            }
        }
        DialogType::Choice => {
            let choices = [
                (dialog.next_dialog_id1, "[1]"),
                (dialog.next_dialog_id2, "[2]"),
                (dialog.next_dialog_id3, "[3]"),
            ];

            for (next_id, label) in choices.iter() {
                if let Some(next) = next_id {
                    if *next != 0 {
                        println!("{}└─ {}─ ", "   ".repeat(depth), label);
                        print_node_recursive(
                            dialog_map,
                            texts,
                            npcs,
                            event_info,
                            *next,
                            depth + 1,
                            printed,
                        );
                    }
                }
            }
        }
    }
}

/// Find entry points - dialogs that are not referenced by any other dialog
fn find_entry_points(
    dialog_map: &HashMap<i32, &DialogueScript>,
    all_ids: &HashSet<i32>,
) -> Vec<i32> {
    let mut entry_points = Vec::new();

    for &id in all_ids {
        if id == 0 {
            continue;
        }
        if is_entry_point(dialog_map, id) {
            entry_points.push(id);
        }
    }

    // If no entry points found, use first few dialogs
    if entry_points.is_empty() {
        for &id in all_ids.iter().filter(|&&x| x != 0).take(10) {
            entry_points.push(id);
        }
    }

    entry_points.sort();
    entry_points
}
