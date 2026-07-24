// dispel-gui-win32 library crate.
// Provides the Win32-native GUI application shell and editor framework.
// This crate compiles only on Windows.

#![cfg(target_os = "windows")]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_imports)]

pub mod app;
pub mod shell;
pub mod editors;
pub mod file_tree;
pub mod spreadsheet;
pub mod text_editor;
pub mod hex_editor;
pub mod canvas;
pub mod audio;
pub mod db_viewer;
pub mod mod_packager;
pub mod localization;
pub mod save_viewer;
pub mod dialogue_editor;
pub mod undo_redo;
pub mod search;
pub mod theme;
pub mod native_dialogs;
pub mod style;
