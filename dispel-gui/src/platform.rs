//! Cross-platform utilities.

use std::path::Path;

/// Open the containing folder of `path` in the OS file manager and (where
/// supported) select the file.
///
/// - **Windows**: `explorer /select,<path>`
/// - **macOS**:    `open -R <path>`
/// - **Linux**:    `xdg-open <parent-dir>`
pub fn open_in_file_manager(path: &Path) {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let _ = Command::new("explorer").arg("/select,").arg(path).spawn();
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let _ = Command::new("open").arg("-R").arg(path).spawn();
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        if let Some(parent) = path.parent() {
            let _ = Command::new("xdg-open").arg(parent).spawn();
        }
    }
}

/// Search for an SNF audio file in the game directory or a `Sound/` subdirectory,
/// falling back to a recursive subdirectory search.
pub fn find_snf_file(game_path: &str, snf_filename: &str) -> std::path::PathBuf {
    let direct = std::path::PathBuf::from(game_path).join(snf_filename);
    if direct.exists() {
        return direct;
    }

    let candidate = std::path::PathBuf::from(game_path).join("Sound").join(snf_filename);
    if candidate.exists() {
        return candidate;
    }

    if let Ok(entries) = std::fs::read_dir(game_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let candidate = path.join(snf_filename);
                if candidate.exists() {
                    return candidate;
                }
            }
        }
    }
    direct
}
