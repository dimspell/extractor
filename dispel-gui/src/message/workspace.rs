use iced::widget::pane_grid;

#[derive(Debug, Clone)]
pub enum WorkspaceMessage {
    FileTree(crate::components::file_tree::message::FileTreeMessage),
    TabBar(crate::components::tab_bar::TabBarMessage),
    ToggleSidebar,
    ToggleCommandPalette,
    ToggleGlobalSearch,
    ToggleHistoryPanel,
    // ToggleAutoSave, // Implemented in system
    ToggleMaximizePane,
    CommandPaletteInput(String),
    CommandPaletteSelect(usize),
    CommandPaletteClose,
    CommandPaletteConfirm,
    CommandPaletteArrowUp,
    CommandPaletteArrowDown,
    GlobalSearchInput(String),
    GlobalSearchSelect(usize),
    GlobalSearchConfirm,
    GlobalSearchArrowUp,
    GlobalSearchArrowDown,
    GlobalSearchAsync(String),
    OpenToolTab(crate::workspace::EditorType),
    PaneClicked(pane_grid::Pane),
    PaneDragged(pane_grid::DragEvent),
    PaneResized(pane_grid::ResizeEvent),
}
