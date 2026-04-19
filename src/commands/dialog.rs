use super::Command;
use dispel_core::references::dialog::{read_dialogs, Dialog};
use dispel_core::references::dialogue_text::read_dialogue_texts;
use dispel_core::references::npc_ref::read_npc_ref;
use dispel_core::DialogType;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Clone)]
struct NpcInfo {
    name: String,
    description: String,
}

pub struct DialogCommand {
    pub dlg_path: String,
    pub pgp_path: Option<String>,
    pub npc_ref_path: Option<String>,
}

impl Command for DialogCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        let dlg_path = Path::new(&self.dlg_path);
        let dialogs =
            read_dialogs(dlg_path).map_err(|e| format!("ERROR: could not read DLG file: {e}"))?;

        let texts: HashMap<i32, String> = if let Some(pgp_path) = &self.pgp_path {
            let pgp_path = Path::new(pgp_path);
            let dialogue_texts = read_dialogue_texts(pgp_path)
                .map_err(|e| format!("ERROR: could not read PGP file: {e}"))?;
            dialogue_texts.into_iter().map(|t| (t.id, t.text)).collect()
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

        print_dialog_flow(&dialogs, &texts, &npcs);
        Ok(())
    }
}

fn print_dialog_flow(
    dialogs: &[Dialog],
    texts: &HashMap<i32, String>,
    npcs: &HashMap<i32, NpcInfo>,
) {
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                           DIALOG FLOW                                         ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();

    let dialog_map: HashMap<i32, &Dialog> = dialogs.iter().map(|d| (d.id, d)).collect();

    let mut printed = HashMap::new();
    let mut entry_points = Vec::new();

    for dialog in dialogs {
        if dialog.id == 0 {
            continue;
        }
        if is_entry_point(&dialog_map, dialog.id) {
            entry_points.push(dialog.id);
        }
    }

    if entry_points.is_empty() {
        for dialog in dialogs {
            if dialog.id != 0 && !printed.contains_key(&dialog.id) {
                entry_points.push(dialog.id);
            }
            if entry_points.len() >= 10 {
                break;
            }
        }
    }

    for entry_id in entry_points {
        if printed.contains_key(&entry_id) {
            continue;
        }

        let npc_info = npcs.get(&entry_id);
        let npc_label = if let Some(npc) = npc_info {
            if !npc.name.is_empty() {
                format!("{} ({})", npc.name.trim(), npc.description.trim())
            } else {
                npc.description.trim().to_string()
            }
        } else {
            String::new()
        };

        println!("┌──────────────────────────────────────────────────────────────────────────────");
        if npc_label.is_empty() {
            println!("│ CONVERSATION {} (Entry Point)", entry_id);
        } else {
            println!("│ CONVERSATION {} (Entry Point) - {}", entry_id, npc_label);
        }
        println!("└──────────────────────────────────────────────────────────────────────────────");
        print_node_recursive(&dialog_map, texts, entry_id, 0, &mut printed);
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
            println!(
                "└─ DLG {} (speaker: {:?}, type: {:?})",
                dialog.id, dialog.dialog_owner, dialog.dialog_type
            );
        }
        println!();
    }

    if printed.is_empty() {
        println!("No dialogs found. This file may not contain any conversation entries.");
    }
}

fn is_entry_point(dialog_map: &HashMap<i32, &Dialog>, id: i32) -> bool {
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
    dialog_map: &HashMap<i32, &Dialog>,
    texts: &HashMap<i32, String>,
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

    println!(
        "{}┌─ DLG {} {} (speaker: {})",
        "   ".repeat(depth),
        id,
        type_str,
        owner
    );
    if !text.is_empty() {
        let lines: Vec<&str> = text.lines().collect();
        for line in &lines {
            println!("{}│   \"{}\"", "   ".repeat(depth), line);
        }
    }
    if let Some(event_id) = dialog.triggered_event_id {
        if event_id != 0 {
            println!("{}│   └─ Triggers Event: {}", "   ".repeat(depth), event_id);
        }
    }
    if let Some(req_event) = dialog.required_event_id {
        if req_event != 0 {
            println!(
                "{}│   └─ Requires Event: {}",
                "   ".repeat(depth),
                req_event
            );
        }
    }

    match dialog_type {
        DialogType::Normal => {
            if let Some(next) = dialog.next_dialog_id1 {
                if next != 0 {
                    println!("{}│", "   ".repeat(depth));
                    print_node_recursive(dialog_map, texts, next, depth, printed);
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
                        print_node_recursive(dialog_map, texts, *next, depth + 1, printed);
                    }
                }
            }
        }
    }
}
