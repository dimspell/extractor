pub mod encoding_modal;
pub mod export_modal;
pub mod extend_modal;
pub mod fill_modal;
pub mod footer;
pub mod goto_modal;
pub mod inspector;
pub mod inspector_modal;
pub mod matrix;
pub mod minimap;
pub mod panel;
pub mod patterns;
pub mod repeat_modal;
pub mod search_overlay;
pub mod settings_modal;
pub mod statistics;
pub mod toolbar;

pub(crate) mod diff;

use gui_widgets::components::context_menu::{ContextMenu, Entry as MenuEntry};
use gui_widgets::components::modal::modal;
use gui_widgets::components::toast;
use iced::widget::pane_grid;
use iced::widget::pane_grid::PaneGrid;
use iced::widget::space::Space;
use iced::widget::{column, container, text};
use iced::{Element, Fill, Font};

use crate::config::HexEditorConfig;
use crate::domain::panel::HexPanelContent;
use crate::{HexEditorMessage, HexEditorState, HexProvider};

use self::toolbar::build_toolbar;

pub fn view<'a>(
    state: &'a HexEditorState,
    config: &HexEditorConfig,
) -> Element<'a, HexEditorMessage> {
    if let Some(ref err) = state.error {
        return container(
            column![
                text("Failed to load file").size(14),
                text(err.as_str()).size(12).font(Font::MONOSPACE),
            ]
            .spacing(8),
        )
        .width(Fill)
        .height(Fill)
        .padding(16)
        .into();
    }

    let total = state.provider.len();
    let header = container(
        text(format!(
            "{}  ·  {} bytes  ·  {} bytes/row",
            state.name, total, state.bytes_per_row
        ))
        .size(11)
        .font(Font::MONOSPACE),
    )
    .padding([6, 12])
    .width(Fill);

    let toolbar = build_toolbar(state, config);

    // ── Halloy-style Pane Grid ──────────────────────────────────────────
    // The body area is a split-pane grid with movable/resizable panels.
    // Each panel contains one sub-view (matrix, inspector, pattern list).
    // Follows Halloy's pattern: spacing between panes, outer padding, and
    // focused-pane highlighting in the title bar.
    let pane_count = state.panes.len();
    let pane_focus = state.pane_focus;
    let pane_grid = PaneGrid::new(
        &state.panes,
        |id, panel: &crate::domain::panel::HexPanel, _maximized| {
            let is_focused = id == pane_focus;

            // Both matrix and diff panes get a context menu for pattern
            // and diff operations (Create/Remove pattern, Diff Against File,
            // Close Diff, etc.).
            let content = match panel.content {
                HexPanelContent::Matrix | HexPanelContent::Diff => {
                    let inner = panel::pane_content(state, config, id, panel);

                    let context_addr = state.context_menu_addr;
                    let have_pattern_at_addr = context_addr
                        .and_then(|addr| state.pattern_id_at(addr))
                        .is_some();
                    let pattern_at_cursor = context_addr
                        .and_then(|addr| state.pattern_id_at(addr))
                        .and_then(|pid| state.pattern_by_id(pid));
                    let pattern_group_at_cursor = pattern_at_cursor
                        .and_then(|p| p.group_id)
                        .and_then(|gid| state.groups.iter().find(|g| g.id == gid));
                    let group_id_at_cursor = pattern_group_at_cursor.map(|g| g.id);
                    let entries = build_pattern_menu_entries(
                        !state.selection.is_single(),
                        !state.patterns.is_empty(),
                        have_pattern_at_addr,
                        group_id_at_cursor,
                        state.comparison_file.is_some(),
                    );
                    ContextMenu::new(inner, entries).into()
                }
                _ => panel::pane_content(state, config, id, panel),
            };

            pane_grid::Content::new(content)
                .title_bar(panel::title_bar(state, id, panel, pane_count, is_focused))
        },
    )
    .on_click(HexEditorMessage::PaneClicked)
    .on_drag(HexEditorMessage::PaneDragged)
    .on_resize(6, HexEditorMessage::PaneResized)
    .spacing(config.pane_gap as f32)
    .width(Fill)
    .height(Fill);

    let search_section: Element<'a, HexEditorMessage> = if state.search.is_visible() {
        search_overlay::view(&state.search, state.theme)
    } else {
        Space::default().height(0).into()
    };

    // Halloy-style outer padding around the pane grid.
    let pane_grid = container(pane_grid)
        .padding(config.pane_gap as f32)
        .width(Fill)
        .height(Fill);

    let base: Element<'a, HexEditorMessage> = column![
        toolbar,
        search_section,
        header,
        pane_grid,
        footer::view(state),
    ]
    .spacing(0)
    .width(Fill)
    .height(Fill)
    .into();

    let base = if let Some(ref ie) = state.inspector_edit {
        modal(
            base,
            inspector_modal::view(ie, state.theme),
            || HexEditorMessage::CloseInspectorEdit,
            0.4,
        )
    } else {
        base
    };

    let base = if let Some(ref g) = state.goto {
        modal(
            base,
            goto_modal::view(g, state.theme),
            || HexEditorMessage::CloseGotoDialog,
            0.3,
        )
    } else {
        base
    };

    let mut base = if let Some(ref rp) = state.repeat_pattern {
        modal(
            base,
            repeat_modal::view(rp, state.theme),
            || HexEditorMessage::CloseRepeatedPattern,
            0.3,
        )
    } else {
        base
    };

    if state.export_config.is_some() {
        base = modal(
            base,
            export_modal::view(state),
            || HexEditorMessage::CloseExportConfig,
            0.35,
        );
    }

    if state.settings_open {
        base = modal(
            base,
            settings_modal::view(state),
            || HexEditorMessage::CloseSettings,
            0.35,
        );
    }

    if state.encoding_settings_open {
        base = modal(
            base,
            encoding_modal::view(state),
            || HexEditorMessage::CloseEncodingSettings,
            0.4,
        );
    }

    if let Some(ref dlg) = state.fill_dialog {
        base = modal(
            base,
            fill_modal::view(dlg, state.theme),
            || HexEditorMessage::CloseFill,
            0.3,
        );
    }

    if let Some(ref dlg) = state.extend_dialog {
        base = modal(
            base,
            extend_modal::view(dlg, state.theme),
            || HexEditorMessage::CloseExtend,
            0.3,
        );
    }

    toast::Manager::new(
        base,
        &state.notifications,
        HexEditorMessage::DismissNotification,
    )
    .into()
}

/// Build the pattern menu entries for the context menu.
///
/// Extracted as a pure function so the enabled/disabled logic can be
/// unit-tested without simulating UI interactions.
///
/// - `has_selection_range`: `true` if there is a multi-byte selection (enables
///   "Create Pattern" and "Add Repeated Pattern")
/// - `has_patterns`: `true` if any patterns exist
/// - `have_pattern_at_addr`: `true` if the right-click address falls within a
///   pattern (enables "Remove Pattern")
pub(crate) fn build_pattern_menu_entries(
    has_selection_range: bool,
    has_patterns: bool,
    have_pattern_at_addr: bool,
    group_id_at_cursor: Option<usize>,
    has_comparison: bool,
) -> Vec<MenuEntry<HexEditorMessage>> {
    let mut entries = Vec::new();
    // ── Diff actions ───────────────────────────────────────────────────
    if has_selection_range {
        entries.push(MenuEntry::item(
            "Create Pattern",
            HexEditorMessage::CreatePattern,
        ));
        entries.push(MenuEntry::item(
            "Add Repeated Pattern",
            HexEditorMessage::BeginRepeatedPattern,
        ));
    } else {
        entries.push(MenuEntry::disabled("Create Pattern"));
        entries.push(MenuEntry::disabled("Add Repeated Pattern"));
    }
    if has_patterns {
        if have_pattern_at_addr {
            entries.push(MenuEntry::item(
                "Remove Pattern",
                HexEditorMessage::RemovePatternAtContextMenu,
            ));
        } else {
            entries.push(MenuEntry::disabled("Remove Pattern"));
        }
        if let Some(gid) = group_id_at_cursor {
            entries.push(MenuEntry::separator());
            entries.push(MenuEntry::item(
                "Remove Group",
                HexEditorMessage::RemovePatternGroup(gid),
            ));
        }
    }
    if has_patterns {
        entries.push(MenuEntry::item(
            "Clear All Patterns",
            HexEditorMessage::ClearAllPatterns,
        ));
    } else {
        entries.push(MenuEntry::disabled("Clear All Patterns"));
    }
    entries.push(MenuEntry::separator());
    if has_patterns {
        entries.push(MenuEntry::item(
            "Export Patterns…",
            HexEditorMessage::ExportPatterns,
        ));
    } else {
        entries.push(MenuEntry::disabled("Export Patterns…"));
    }
    entries.push(MenuEntry::item(
        "Import Patterns…",
        HexEditorMessage::ImportPatterns,
    ));
    entries.push(MenuEntry::separator());
    if has_selection_range {
        entries.push(MenuEntry::item("Fill…", HexEditorMessage::BeginFill));
    } else {
        entries.push(MenuEntry::disabled("Fill…"));
    }
    // "Extend…" needs no selection — a single cursor suffices (the context
    // menu only appears on a right-click over a byte).
    entries.push(MenuEntry::item("Extend…", HexEditorMessage::BeginExtend));
    entries.push(MenuEntry::separator());

    if has_comparison {
        entries.push(MenuEntry::item(
            "Close Diff",
            HexEditorMessage::CloseComparison,
        ));
    } else {
        entries.push(MenuEntry::item(
            "Diff Against File…",
            HexEditorMessage::LoadComparisonFile,
        ));
    }
    entries.push(MenuEntry::separator());

    entries.push(MenuEntry::item("Settings", HexEditorMessage::OpenSettings));
    entries.push(MenuEntry::item("Toggle Inspector pane", HexEditorMessage::ToggleInspector));
    entries.push(MenuEntry::item("Toggle Patterns list", HexEditorMessage::TogglePatternList));
    entries.push(MenuEntry::item("Toggle Stats pane", HexEditorMessage::ToggleStats));
    entries.push(MenuEntry::item("Export TXT", HexEditorMessage::OpenExportConfig));
    entries
}
