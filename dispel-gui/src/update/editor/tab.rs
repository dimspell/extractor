use std::collections::HashMap;
use std::path::PathBuf;

use crate::components::editable::EditableRecord;
use crate::components::generic_editor::TabbedEditor;
use crate::components::item_catalog::ensure_item_lookups;
use crate::components::loading_state::LoadingState;
use crate::message::Message;
use crate::workspace::{TabId, Workspace};
use dispel_core::Extractor;
use iced::Task;
use iced::widget::pane_grid;

/// Lookup keys used by the tabbed editors' display caches.
///
/// Defined once here so every producer (monster/NPC name loaders) and every
/// consumer (`Lookup(...)` field descriptors) agrees on the exact key.
pub mod lookup_keys {
    pub const MONSTER_NAMES: &str = "monster_names";
    pub const NPC_NAMES: &str = "NPC";
}

pub fn get_tab_id(workspace: &Workspace) -> TabId {
    workspace.active_tab_id()
}

pub fn select<T: EditableRecord + Extractor>(
    tabbed_editor: &mut TabbedEditor<T>,
    tab_id: usize,
    index: usize,
) -> Task<Message> {
    if let Some(editor) = tabbed_editor.editors.get_mut(&tab_id) {
        editor.select(index);
    }
    Task::none()
}

pub fn field_changed<T: EditableRecord + Extractor>(
    tabbed_editor: &mut TabbedEditor<T>,
    tab_id: usize,
    index: usize,
    field: String,
    value: String,
) -> Task<Message> {
    if let Some(editor) = tabbed_editor.editors.get_mut(&tab_id) {
        editor.update_field(index, &field, value);
    }
    Task::none()
}

pub fn save<T: EditableRecord + Extractor>(
    tabbed_editor: &mut TabbedEditor<T>,
    tab_id: usize,
    success_msg: &str,
    error_label: &str,
) -> Task<Message> {
    if let Some(editor) = tabbed_editor.editors.get_mut(&tab_id) {
        editor.editor.loading_state = LoadingState::Loading;
        let result = editor.save();
        editor.editor.loading_state = LoadingState::Loaded(());
        match result {
            Ok(_) => editor.editor.status_msg = success_msg.into(),
            Err(e) => editor.editor.status_msg = format!("{}: {}", error_label, e),
        }
    }
    Task::none()
}

pub fn add_entry<T: EditableRecord + Extractor>(
    tabbed_editor: &mut TabbedEditor<T>,
    tab_id: usize,
) -> Task<Message> {
    if let Some(editor) = tabbed_editor.editors.get_mut(&tab_id) {
        editor.add_record();
    }
    Task::none()
}

pub fn remove_entry<T: EditableRecord + Extractor>(
    tabbed_editor: &mut TabbedEditor<T>,
    tab_id: usize,
    index: usize,
) -> Task<Message> {
    if let Some(editor) = tabbed_editor.editors.get_mut(&tab_id)
        && let Err(msg) = editor.remove_record(index)
    {
        eprintln!("{}", msg);
    }
    Task::none()
}

pub fn pane_resized<T: EditableRecord>(
    tabbed_editor: &mut TabbedEditor<T>,
    tab_id: usize,
    event: pane_grid::ResizeEvent,
) -> Task<Message> {
    if let Some(editor) = tabbed_editor.editors.get_mut(&tab_id)
        && let Some(ref mut ps) = editor.editor.pane_state
    {
        ps.resize(event.split, event.ratio);
    }
    if let Some(ss) = tabbed_editor.spreadsheets.get_mut(&tab_id)
        && let Some(ref mut ps) = ss.pane_state
    {
        ps.resize(event.split, event.ratio);
    }
    Task::none()
}

pub fn pane_clicked<T: EditableRecord>(
    tabbed_editor: &mut TabbedEditor<T>,
    tab_id: usize,
    pane: pane_grid::Pane,
) -> Task<Message> {
    if let Some(editor) = tabbed_editor.editors.get_mut(&tab_id) {
        editor.editor.pane_focus = Some(pane);
    }
    Task::none()
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared catalog-load flow for every tabbed editor
// ─────────────────────────────────────────────────────────────────────────────
//
// These two functions are the *only* place a tabbed editor parses a catalog
// file and installs it into editor + spreadsheet state.  The tab id is
// captured when the task is spawned and carried in the completion message, so
// a tab switch while the read is in flight cannot misdirect the result.

/// Prepare a tab's editor/spreadsheet state and start a single asynchronous
/// catalog read.  The read result is routed back via `wrap(tab_id, result)`.
///
/// The path is stored on the editor immediately so the UI can show which file
/// is loading; the file itself is parsed exactly once (in the async task).
pub fn load_catalog<R, F>(
    path: PathBuf,
    tabbed: &mut TabbedEditor<R>,
    tab_id: usize,
    game_path: &str,
    lookups: &mut HashMap<String, Vec<(String, String)>>,
    wrap: F,
) -> Task<Message>
where
    R: EditableRecord + Extractor + Clone + std::fmt::Debug + Send + 'static,
    F: Fn(usize, Result<Vec<R>, String>) -> Message + Send + 'static,
{
    // Composite-item fields need the item lookup before caches are built.
    ensure_item_lookups(game_path, lookups);

    {
        let editor = tabbed.editors.entry(tab_id).or_default();
        editor.current_file = Some(path.clone());
        editor.editor.loading_state = LoadingState::Loading;
    }
    tabbed.spreadsheets.entry(tab_id).or_default();

    Task::perform(
        async move { R::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
        move |result| wrap(tab_id, result),
    )
}

/// Apply a completed catalog read to the tab it was started for.
///
/// `tab_id` comes from the message (captured at spawn time), *not* from the
/// currently active tab.  If that tab was closed or its editor state was
/// cleared in the meantime the result is safely dropped.
pub fn catalog_loaded<R>(
    tab_id: usize,
    result: Result<Vec<R>, String>,
    tabbed: &mut TabbedEditor<R>,
    lookups: &HashMap<String, Vec<(String, String)>>,
    loaded_prefix: &str,
    error_prefix: &str,
) where
    R: EditableRecord + Extractor + Clone + std::fmt::Debug,
{
    let editors = &mut tabbed.editors;
    let spreadsheets = &mut tabbed.spreadsheets;
    let Some(editor) = editors.get_mut(&tab_id) else {
        // The tab was switched away from and its state removed — drop the
        // stale result instead of writing it into some unrelated tab.
        return;
    };

    editor.editor.loading_state = LoadingState::Loaded(());
    match result {
        Ok(catalog) => {
            editor.editor.status_msg = format!("{}{} entries", loaded_prefix, catalog.len());
            editor.editor.catalog = Some(catalog.clone());
            editor.editor.refresh();
            if let Some(ss) = spreadsheets.get_mut(&tab_id) {
                ss.apply_filter(&catalog);
                ss.compute_all_caches(&catalog, lookups);
                ss.init_pane_state();
            }
        }
        Err(e) => {
            editor.editor.status_msg = format!("{}{}", error_prefix, e);
        }
    }
}

/// Rebuild display caches for every open tab of a tabbed editor.
///
/// Used when a shared lookup (e.g. monster/NPC names) arrives after one or
/// more catalogs have already been loaded, so all tabs stay consistent.
pub fn rebuild_all_caches<R>(
    tabbed: &mut TabbedEditor<R>,
    lookups: &HashMap<String, Vec<(String, String)>>,
) where
    R: EditableRecord + Extractor + Clone + std::fmt::Debug,
{
    let editors = &tabbed.editors;
    let spreadsheets = &mut tabbed.spreadsheets;
    for (id, ss) in spreadsheets.iter_mut() {
        if let Some(editor) = editors.get(id)
            && let Some(catalog) = editor.editor.catalog.as_ref()
        {
            ss.compute_all_caches(catalog, lookups);
        }
    }
}

/// Read an INI file asynchronously and map its records to `(id, name)` pairs
/// for the shared lookup table.  `wrap` routes the result to the caller's
/// editor-specific message variant.
pub fn load_name_lookup<R, M, F>(path: PathBuf, to_entries: F, wrap: M) -> Task<Message>
where
    R: Extractor + Send + 'static,
    F: Fn(Vec<R>) -> Vec<(String, String)> + Send + 'static,
    M: Fn(Result<Vec<(String, String)>, String>) -> Message + Send + 'static,
{
    Task::perform(
        async move {
            R::read_file(&path)
                .map(to_entries)
                .map_err(|e: std::io::Error| e.to_string())
        },
        wrap,
    )
}
