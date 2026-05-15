use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use dispel_core::references::event_scr::EventScript;

/// Index of all event-script functions discovered across the game's
/// `Ref/Event*.scr` files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventScriptFunctionIndex {
    pub functions: Vec<IndexedFunction>,
    pub scanned_file_count: usize,
    pub scanned_at: i64,
}

/// One distinct `(function_name, param_count)` combo with occurrence count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedFunction {
    pub name: String,
    pub param_count: usize,
    pub frequency: u32,
}

/// Shared mutable progress state, visible from both the background scanner
/// and the Iced event loop (view/update).
#[derive(Debug)]
pub struct IndexProgress {
    pub processed: AtomicU32,
    pub total: AtomicU32,
    pub current_file: Mutex<String>,
    pub cancelled: AtomicBool,
}

impl IndexProgress {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            processed: AtomicU32::new(0),
            total: AtomicU32::new(0),
            current_file: Mutex::new(String::new()),
            cancelled: AtomicBool::new(false),
        })
    }
}

/// Path where the function index JSON is persisted (relative to the workspace
/// directory that contains `workspace.json`).
pub fn index_file_path() -> std::path::PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("../.."));
    path.push("..");
    path.push("indexes");
    std::fs::create_dir_all(&path).ok();
    path.push("event_scr_functions.json");
    path
}

/// Scan all `Event*.scr` files under `game_path/Ref/` and build a
/// frequency-sorted function index.  Reports progress (and checks the
/// cancellation flag) through `progress`.
pub fn build_index(
    game_path: &Path,
    progress: Arc<IndexProgress>,
) -> Result<EventScriptFunctionIndex, String> {
    let ref_dir = game_path.join("Ref");
    if !ref_dir.exists() {
        return Err("Ref/ directory not found under game path".to_string());
    }

    let mut files: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&ref_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());
            if ext.as_deref() != Some("scr") {
                continue;
            }
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str());
            if stem.is_some_and(|s| s.to_lowercase().starts_with("event")) {
                files.push(path);
            }
        }
    }

    progress.total.store(files.len() as u32, Ordering::Relaxed);

    let mut counter: HashMap<(String, usize), u32> = HashMap::new();

    for file in &files {
        if progress.cancelled.load(Ordering::Relaxed) {
            return Err("Cancelled".to_string());
        }

        *progress.current_file.lock().unwrap() = file
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if let Ok(scripts) = <EventScript as dispel_core::Extractor>::read_file(file) {
            for script in &scripts {
                for action in &script.actions {
                    if action.raw_content.is_none() && !action.function_name.is_empty() {
                        let key = (action.function_name.clone(), action.parameters.len());
                        *counter.entry(key).or_insert(0) += 1;
                    }
                }
            }
        }

        progress.processed.fetch_add(1, Ordering::Relaxed);
    }

    let mut functions: Vec<IndexedFunction> = counter
        .into_iter()
        .map(|((name, param_count), frequency)| IndexedFunction {
            name,
            param_count,
            frequency,
        })
        .collect();

    functions.sort_by_key(|a| std::cmp::Reverse(a.frequency));

    let scanned_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    Ok(EventScriptFunctionIndex {
        functions,
        scanned_file_count: files.len(),
        scanned_at,
    })
}
