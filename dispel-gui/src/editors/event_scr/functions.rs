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
    /// Files whose names matched the scan pattern (including unreadable ones).
    pub scanned_file_count: usize,
    /// Files that were actually parsed and contributed functions.
    #[serde(default)]
    pub parsed_file_count: usize,
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

/// Path where the function index JSON is persisted under the dispel-gui config
/// directory (alongside `workspace.json`).
pub fn index_file_path() -> std::path::PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let dir = base.join("dispel-gui").join("indexes");
    std::fs::create_dir_all(&dir).ok();
    dir.join("event_scr_functions.json")
}

/// Persist the index, returning a description of any serialization or I/O
/// failure so callers can surface it instead of silently dropping the index.
pub fn save_index(index_path: &Path, index: &EventScriptFunctionIndex) -> Result<(), String> {
    let json = serde_json::to_string_pretty(index).map_err(|e| format!("serialize index: {e}"))?;
    std::fs::write(index_path, &json)
        .map_err(|e| format!("write index {}: {e}", index_path.display()))
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
            let stem = path.file_stem().and_then(|s| s.to_str());
            if stem.is_some_and(|s| s.to_lowercase().starts_with("event")) {
                files.push(path);
            }
        }
    }

    progress.total.store(files.len() as u32, Ordering::Relaxed);

    let (counter, parsed_file_count) = index_files(&files, &progress, |file| {
        <EventScript as dispel_core::Extractor>::read_file(file)
    })?;

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
        parsed_file_count,
        scanned_at,
    })
}

type FunctionCounts = HashMap<(String, usize), u32>;

/// Count functions across the candidate files, tracking only files that were
/// actually parsed. A file that fails to parse is skipped, never counted as a
/// successful scan.
fn index_files<F>(
    files: &[std::path::PathBuf],
    progress: &IndexProgress,
    mut read: F,
) -> Result<(FunctionCounts, usize), String>
where
    F: FnMut(&Path) -> std::io::Result<Vec<EventScript>>,
{
    let mut counter: FunctionCounts = HashMap::new();
    let mut parsed_file_count = 0;

    for file in files {
        if progress.cancelled.load(Ordering::Relaxed) {
            return Err("Cancelled".to_string());
        }

        *progress.current_file.lock().unwrap() = file
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if let Ok(scripts) = read(file) {
            parsed_file_count += 1;
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

    Ok((counter, parsed_file_count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dispel_core::references::event_scr::{ActionFunction, EventScript};
    use std::ffi::OsStr;

    fn script_calling(name: &str) -> Vec<EventScript> {
        let mut script = EventScript::default();
        script.actions.push(ActionFunction {
            prefix: None,
            function_name: name.to_string(),
            parameters: vec!["1".to_string()],
            raw_content: None,
        });
        vec![script]
    }

    #[test]
    fn test_index_files_counts_only_parsed_files() {
        let files = vec![
            std::path::PathBuf::from("EventWidget.scr"),
            std::path::PathBuf::from("EventGadget.scr"),
        ];
        let progress = IndexProgress::new();
        let (counter, parsed) = index_files(&files, &progress, |file| {
            if file.file_name() == Some(OsStr::new("EventGadget.scr")) {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "unparseable script",
                ))
            } else {
                Ok(script_calling("invented_call"))
            }
        })
        .expect("indexing must tolerate one unparseable file");

        assert_eq!(parsed, 1, "only the parsed file may be counted");
        assert_eq!(counter.get(&("invented_call".to_string(), 1)), Some(&1));
        assert_eq!(progress.processed.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_save_index_reports_write_error() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("missing-dir").join("index.json");
        let index = EventScriptFunctionIndex {
            functions: vec![],
            scanned_file_count: 0,
            parsed_file_count: 0,
            scanned_at: 0,
        };
        let err = save_index(&path, &index).expect_err("a missing parent directory must fail");
        assert!(err.contains("write index"), "unexpected error: {err}");
    }
}
