//! Integration tests for dispel-gui organized by domain.
//! Each submodule covers a specific area of the application.

use crate::app::App;
use crate::workspace::{EditorType, Workspace, WorkspaceTab};

/// Create an App with a single tab of the given editor type.
/// No actual editor state is loaded — simulates a freshly-opened tab.
pub(crate) fn app_with_tab(editor_type: EditorType) -> App {
    let mut workspace = Workspace::new();
    workspace.tabs.push(WorkspaceTab {
        id: 1,
        label: format!("{:?}", editor_type),
        path: None,
        editor_type,
        modified: false,
        pinned: false,
    });
    workspace.active_tab = Some(0);
    App::test_new(workspace)
}

pub(crate) mod capability_crosscheck;
pub(crate) mod clear_all;
pub(crate) mod command_palette;
pub(crate) mod common;
pub(crate) mod editor_field_edit;
pub(crate) mod editor_registry;
pub(crate) mod error_dialog;
pub(crate) mod file_tree;
pub(crate) mod fog_data;
pub(crate) mod generic_editor_edge;
pub(crate) mod global_search;
pub(crate) mod hex_inspector_toggle;
pub(crate) mod indexation;
pub(crate) mod map_editor;
pub(crate) mod message_routing;
pub(crate) mod pane_grid;
pub(crate) mod save_dispatch;
pub(crate) mod snf_editor;
pub(crate) mod sprite_viewer;
pub(crate) mod start_page;
pub(crate) mod system_messages;
pub(crate) mod tabbed_editor;
pub(crate) mod undo_redo;
pub(crate) mod view_dispatch;
pub(crate) mod workspace;
pub(crate) mod workspace_handler;
pub(crate) mod workspace_unit;
