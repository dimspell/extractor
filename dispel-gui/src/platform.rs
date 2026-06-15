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
