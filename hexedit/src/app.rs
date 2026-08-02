//! Multi-document container for the hex editor: a tab bar + N open documents.
//!
//! This is an opt-in layer that renders the reusable
//! [`gui_widgets::components::tab_bar::TabBar`] widget above a stack of
//! [`HexEditorState`] documents. It exists so the standalone binaries can open
//! several files at once without changing the single-document
//! [`HexEditorState`], [`crate::update`] or [`crate::view`] API consumed by
//! `dispel-gui`.

use std::path::PathBuf;

use gui_widgets::components::context_menu::{platform, Entry};
use gui_widgets::components::tab_bar::{Tab, TabBar, TabBarEvent};
use iced::{Element, Task};

use crate::{update, view, HexEditorConfig, HexEditorMessage, HexEditorState};

/// One open document: an editor state plus its tab-level metadata.
pub struct HexEditorDocument {
    pub state: HexEditorState,
    pub pinned: bool,
}

/// A tabbed collection of [`HexEditorDocument`]s.
pub struct HexEditorApp {
    pub documents: Vec<HexEditorDocument>,
    pub active_tab: Option<usize>,
}

impl HexEditorApp {
    /// Create an empty container (no documents, no active tab).
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
            active_tab: None,
        }
    }
}

impl Default for HexEditorApp {
    fn default() -> Self {
        Self::new()
    }
}

/// Messages produced by the container layer.
#[derive(Debug, Clone)]
pub enum AppMessage {
    /// Open the given files as new documents (appended, last one activated).
    OpenFiles(Vec<PathBuf>),
    /// Activate the document at the given index.
    SelectTab(usize),
    /// Close the document at the given index.
    CloseTab(usize),
    /// Close every document except the one at the given index.
    CloseOthers(usize),
    /// Close every document.
    CloseAll,
    /// Flip the pinned flag of the document at the given index.
    TogglePin(usize),
    /// Move the document at `from` so it lands at insertion gap `to`.
    MoveTab(usize, usize),
    /// No-op — fallback for right-click menus on unsupported platforms and
    /// abandoned drags.
    CancelDrag,
    /// Route a single-document message to the document at the given index.
    Document(usize, HexEditorMessage),
}

/// Update the container state, delegating single-document messages to the
/// per-document [`update`] handler.
pub fn app_update(
    app: &mut HexEditorApp,
    config: &HexEditorConfig,
    message: AppMessage,
) -> Task<AppMessage> {
    match message {
        AppMessage::OpenFiles(paths) => {
            for p in paths {
                app.documents.push(HexEditorDocument {
                    state: HexEditorState::load_from_path(&p),
                    pinned: false,
                });
            }
            if !app.documents.is_empty() {
                app.active_tab = Some(app.documents.len() - 1);
            }
            Task::none()
        }
        AppMessage::SelectTab(i) => {
            if i < app.documents.len() {
                app.active_tab = Some(i);
            }
            Task::none()
        }
        AppMessage::CloseTab(i) => {
            if i < app.documents.len() {
                app.documents.remove(i);
                app.active_tab = match app.active_tab {
                    Some(j) if j == i => {
                        if app.documents.is_empty() {
                            None
                        } else {
                            Some(j.min(app.documents.len() - 1))
                        }
                    }
                    Some(j) if j > i => Some(j - 1),
                    other => other,
                };
            }
            Task::none()
        }
        AppMessage::CloseOthers(i) => {
            if i < app.documents.len() {
                let doc = app.documents.remove(i);
                app.documents.clear();
                app.documents.push(doc);
                app.active_tab = Some(0);
            }
            Task::none()
        }
        AppMessage::CloseAll => {
            app.documents.clear();
            app.active_tab = None;
            Task::none()
        }
        AppMessage::TogglePin(i) => {
            if let Some(doc) = app.documents.get_mut(i) {
                doc.pinned = !doc.pinned;
            }
            Task::none()
        }
        AppMessage::MoveTab(from, to) => {
            let n = app.documents.len();
            if from < n && to <= n && from != to {
                let doc = app.documents.remove(from);
                // Adjust target: if removing left of the original position,
                // shift back (mirrors dispel-gui's CloseTab + insert logic).
                let insert_at = if to > from { to - 1 } else { to };
                app.documents.insert(insert_at, doc);

                // Keep active_tab following the moved document.
                if let Some(active) = app.active_tab {
                    app.active_tab = Some(if active == from {
                        insert_at
                    } else if active > from && active <= insert_at {
                        active - 1
                    } else if active < from && active >= insert_at {
                        active + 1
                    } else {
                        active
                    });
                }
            }
            Task::none()
        }
        AppMessage::CancelDrag => Task::none(),
        AppMessage::Document(i, msg) => {
            let Some(doc) = app.documents.get_mut(i) else {
                return Task::none();
            };
            update(&mut doc.state, config, msg).map(move |m| AppMessage::Document(i, m))
        }
    }
}

/// Render the container: a tab bar above the active document's editor view.
///
/// Returns an empty, full-size container when no document is active.
pub fn app_view<'a>(app: &'a HexEditorApp, config: &HexEditorConfig) -> Element<'a, AppMessage> {
    let Some(active) = app.active_tab else {
        return empty_element();
    };
    if active >= app.documents.len() {
        return empty_element();
    }

    let tabs: Vec<Tab> = app
        .documents
        .iter()
        .enumerate()
        .map(|(i, doc)| {
            Tab::new(i, doc.state.name.clone())
                .modified(doc.state.provider.dirty_count() > 0)
                .pinned(doc.pinned)
        })
        .collect();

    let tab_bar: Element<'_, AppMessage> = TabBar::new(tabs, app.active_tab)
        .on_event(move |event| match event {
            TabBarEvent::Selected(i) => AppMessage::SelectTab(i),
            TabBarEvent::Closed(i) => AppMessage::CloseTab(i),
            TabBarEvent::Dragged(from, to) => AppMessage::MoveTab(from, to),
            TabBarEvent::DragCanceled(_) => AppMessage::CancelDrag,
            TabBarEvent::RightClicked(i) => {
                // Try a native context menu; fall back to a no-op when the
                // platform can't show one (e.g. Linux or cancelled).
                try_native_context_menu(app, i).unwrap_or(AppMessage::CancelDrag)
            }
        })
        .into();

    let content: Element<'_, AppMessage> =
        view(&app.documents[active].state, config).map(move |m| AppMessage::Document(active, m));

    iced::widget::column![tab_bar, content]
        .spacing(0)
        .height(iced::Length::Fill)
        .into()
}

/// Placeholder shown when no document is active.
fn empty_element<'a>() -> Element<'a, AppMessage> {
    iced::widget::container(iced::widget::text(""))
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
}

/// Build the right-click context menu entries for a given tab.
fn context_entries_for_tab(app: &HexEditorApp, tab_idx: usize) -> Vec<Entry<AppMessage>> {
    let pinned = app
        .documents
        .get(tab_idx)
        .map(|d| d.pinned)
        .unwrap_or(false);
    vec![
        Entry::item("Close", AppMessage::CloseTab(tab_idx)),
        Entry::separator(),
        Entry::item("Close Others", AppMessage::CloseOthers(tab_idx)),
        Entry::item("Close All", AppMessage::CloseAll),
        Entry::separator(),
        Entry::item(
            if pinned { "Unpin" } else { "Pin" },
            AppMessage::TogglePin(tab_idx),
        ),
    ]
}

/// Show a native context menu for `tab_idx`, returning the resulting message
/// if the user picked an action, or `None` if cancelled / unavailable.
fn try_native_context_menu(app: &HexEditorApp, tab_idx: usize) -> Option<AppMessage> {
    let entries = context_entries_for_tab(app, tab_idx);
    match platform::try_show_native_menu(&entries) {
        Some(platform::NativeResult::Selected(entry_idx)) => {
            entries.get(entry_idx).and_then(|entry| match entry {
                Entry::Item { action, .. } => Some(action.clone()),
                Entry::Separator | Entry::Disabled { .. } => None,
            })
        }
        Some(platform::NativeResult::Cancelled) | None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(name: &str) -> HexEditorDocument {
        HexEditorDocument {
            state: HexEditorState::from_bytes(name, Vec::new(), None, None),
            pinned: false,
        }
    }

    fn app_with(n: usize) -> HexEditorApp {
        let mut app = HexEditorApp::new();
        for i in 0..n {
            app.documents.push(doc(&format!("doc{i}")));
        }
        if n > 0 {
            app.active_tab = Some(0);
        }
        app
    }

    fn config() -> HexEditorConfig {
        HexEditorConfig::default()
    }

    #[test]
    fn test_new_is_empty() {
        let app = HexEditorApp::new();
        assert!(app.documents.is_empty());
        assert_eq!(app.active_tab, None);
    }

    #[test]
    fn test_open_files_activates_last() {
        let mut app = HexEditorApp::new();
        let paths = vec![PathBuf::from("/tmp/a.bin"), PathBuf::from("/tmp/b.bin")];
        let _ = app_update(&mut app, &config(), AppMessage::OpenFiles(paths));
        assert_eq!(app.documents.len(), 2);
        assert_eq!(app.active_tab, Some(1));
    }

    #[test]
    fn test_select_tab_out_of_bounds_is_noop() {
        let mut app = app_with(2);
        app.active_tab = Some(1);
        let _ = app_update(&mut app, &config(), AppMessage::SelectTab(10));
        assert_eq!(app.active_tab, Some(1));
    }

    #[test]
    fn test_close_active_middle_tab_moves_to_next() {
        let mut app = app_with(3);
        app.active_tab = Some(1);
        let _ = app_update(&mut app, &config(), AppMessage::CloseTab(1));
        assert_eq!(app.documents.len(), 2);
        // Active moves to the tab that shifted into index 1.
        assert_eq!(app.active_tab, Some(1));
        assert_eq!(app.documents[1].state.name, "doc2");
    }

    #[test]
    fn test_close_last_tab_moves_active_back() {
        let mut app = app_with(3);
        app.active_tab = Some(2);
        let _ = app_update(&mut app, &config(), AppMessage::CloseTab(2));
        assert_eq!(app.documents.len(), 2);
        assert_eq!(app.active_tab, Some(1));
    }

    #[test]
    fn test_close_only_tab_clears_active() {
        let mut app = app_with(1);
        app.active_tab = Some(0);
        let _ = app_update(&mut app, &config(), AppMessage::CloseTab(0));
        assert!(app.documents.is_empty());
        assert_eq!(app.active_tab, None);
    }

    #[test]
    fn test_close_others_keeps_given_tab() {
        let mut app = app_with(3);
        app.active_tab = Some(2);
        let _ = app_update(&mut app, &config(), AppMessage::CloseOthers(1));
        assert_eq!(app.documents.len(), 1);
        assert_eq!(app.documents[0].state.name, "doc1");
        assert_eq!(app.active_tab, Some(0));
    }

    #[test]
    fn test_close_all_empties_everything() {
        let mut app = app_with(3);
        let _ = app_update(&mut app, &config(), AppMessage::CloseAll);
        assert!(app.documents.is_empty());
        assert_eq!(app.active_tab, None);
    }

    #[test]
    fn test_move_tab_reorders_and_active_follows() {
        let mut app = app_with(3);
        app.active_tab = Some(0);
        let _ = app_update(&mut app, &config(), AppMessage::MoveTab(0, 2));
        assert_eq!(app.documents[0].state.name, "doc1");
        assert_eq!(app.documents[1].state.name, "doc0");
        assert_eq!(app.documents[2].state.name, "doc2");
        // The active document (doc0) moved to index 1.
        assert_eq!(app.active_tab, Some(1));
    }

    #[test]
    fn test_move_tab_backward_reorders() {
        let mut app = app_with(3);
        app.active_tab = Some(2);
        let _ = app_update(&mut app, &config(), AppMessage::MoveTab(2, 0));
        assert_eq!(app.documents[0].state.name, "doc2");
        assert_eq!(app.documents[1].state.name, "doc0");
        assert_eq!(app.documents[2].state.name, "doc1");
        // Active was the moved tab (index 2 → insert_at 0).
        assert_eq!(app.active_tab, Some(0));
    }

    #[test]
    fn test_document_out_of_range_is_noop() {
        let mut app = app_with(2);
        let _ = app_update(
            &mut app,
            &config(),
            AppMessage::Document(99, HexEditorMessage::ToggleAddrFormat),
        );
        assert_eq!(app.documents.len(), 2);
        assert_eq!(app.active_tab, Some(0));
    }

    #[test]
    fn test_toggle_pin_flips() {
        let mut app = app_with(1);
        let _ = app_update(&mut app, &config(), AppMessage::TogglePin(0));
        assert!(app.documents[0].pinned);
        let _ = app_update(&mut app, &config(), AppMessage::TogglePin(0));
        assert!(!app.documents[0].pinned);
    }
}
