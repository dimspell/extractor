// dispel-gui-win32: Windows-native GUI for the Dispel RPG file extractor.
// This binary is Windows-only and uses native Win32 APIs via the `windows` crate.

#[cfg(target_os = "windows")]
fn main() {
    // Entry point for the Win32 native GUI application.
    // The full application shell (message loop, WndProc, controls) is implemented
    // in the library crate and invoked from here.
    if let Err(e) = dispel_gui_win32::run() {
        eprintln!("Application error: {:?}", e);
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("dispel-gui-win32 is a Windows-only application. Use dispel-gui on other platforms.");
}
