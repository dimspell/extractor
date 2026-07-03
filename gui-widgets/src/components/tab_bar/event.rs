/// Events produced by the [`TabBar`] widget.
///
/// The app maps these to its own message type via the `on_event` closure.
#[derive(Debug, Clone, Copy)]
pub enum TabBarEvent {
    /// A tab was selected (left-clicked).
    Selected(usize),
    /// A tab was closed via middle-click.
    Closed(usize),
    /// A tab was dragged from `usize` to `usize`.
    Dragged(usize, usize),
    /// A drag operation was cancelled (e.g. released outside the bar).
    DragCanceled(usize),
    /// Right-click on a tab — the app should handle a context menu.
    RightClicked(usize),
}
