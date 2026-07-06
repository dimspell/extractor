//! A reusable, generic collapsible tree widget.
//!
//! Provides [`TreeNode<T>`] for tree data and [`CollapsibleTree`] for
//! rendering.  The caller provides a closure that renders each node's
//! content (including toggle buttons, icons, styling, click actions).
//! The widget handles:
//!
//! * Recursive iteration with configurable indentation per depth level.
//! * Expand / collapse rendering — the caller controls the toggle via
//!   [`RenderContext::expanded`] and [`RenderContext::has_children`].
//! * Optional search filtering with ancestor-visibility preservation
//!   (parents of matching descendants stay visible).

use iced::widget::{column, Column};
use iced::Element;

// ── TreeNode ─────────────────────────────────────────────────────────

/// A generic tree node parameterised by a payload type `T`.
///
/// The widget only reads [`expanded`] and [`children`]; the payload
/// `data` is passed through to the rendering closure untouched.
#[derive(Debug, Clone)]
pub struct TreeNode<T> {
    /// Whether children are visible.
    pub expanded: bool,
    /// Child nodes (empty for leaf nodes).
    pub children: Vec<TreeNode<T>>,
    /// Payload (name, path, icon, id, …).
    pub data: T,
}

impl<T> TreeNode<T> {
    /// Build a leaf node.
    pub fn leaf(data: T) -> Self {
        TreeNode {
            expanded: false,
            children: Vec::new(),
            data,
        }
    }

    /// Build a branch node with children.
    pub fn branch(data: T, children: Vec<TreeNode<T>>) -> Self {
        TreeNode {
            expanded: false,
            children,
            data,
        }
    }

    /// Toggle expanded state. Returns the new state.
    pub fn toggle(&mut self) -> bool {
        self.expanded = !self.expanded;
        self.expanded
    }

    /// Recursively find the first node where `pred(&data)` is true.
    pub fn find_mut(&mut self, pred: &impl Fn(&T) -> bool) -> Option<&mut TreeNode<T>> {
        if pred(&self.data) {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_mut(pred) {
                return Some(found);
            }
        }
        None
    }

    /// Pre-order traversal (parent first, then children).
    pub fn for_each(&self, f: &impl Fn(&TreeNode<T>)) {
        f(self);
        for child in &self.children {
            child.for_each(f);
        }
    }
}

// ── RenderContext ────────────────────────────────────────────────────

/// Context passed to the `render_node` closure for each visible node.
#[derive(Debug, Clone, Copy)]
pub struct RenderContext<'a, T> {
    /// The node's payload.
    pub data: &'a T,
    /// Nesting depth (0 = top-level).
    pub depth: usize,
    /// Whether the node is currently expanded.
    pub expanded: bool,
    /// Whether this node has children.
    pub has_children: bool,
}

// ── CollapsibleTree widget ───────────────────────────────────────────

/// A generic collapsible tree.
///
/// The caller owns the [`TreeNode<T>`] data and provides a `render_node`
/// closure that produces the visual content for any visible node.  The
/// closure receives a [`RenderContext`] that includes `depth` (for
/// indentation), `expanded`, and `has_children` (to draw the toggle
/// caret).  The widget handles the recursive walk, indentation, and
/// optional search-filtering with ancestor preservation.
///
/// # Type parameters
///
/// * `T` — the payload stored in each node.
/// * `Message` — the Iced message type emitted by interactions.
/// * `Theme` — the Iced theme type (defaults to `iced::Theme`).
/// * `Renderer` — the Iced renderer (defaults to `iced::Renderer`);
///   must satisfy [`iced::advanced::Renderer`].
pub struct CollapsibleTree<
    'a,
    T,
    Message,
    Theme = iced::Theme,
    Renderer: iced::advanced::Renderer = iced::Renderer,
> {
    nodes: &'a [TreeNode<T>],
    spacing: f32,
    indent: f32,
    #[allow(clippy::type_complexity)]
    filter: Option<Box<dyn Fn(&T) -> bool + 'a>>,
    render_node: Box<
        dyn Fn(RenderContext<'a, T>) -> Element<'a, Message, Theme, Renderer> + 'a,
    >,
}

impl<'a, T, Message: 'a, Theme: 'a + iced::widget::container::Catalog, Renderer: 'a + iced::advanced::Renderer>
    CollapsibleTree<'a, T, Message, Theme, Renderer>
{
    /// Create a new tree over `nodes` with the given rendering closure.
    pub fn new(
        nodes: &'a [TreeNode<T>],
        render_node: impl Fn(RenderContext<'a, T>) -> Element<'a, Message, Theme, Renderer> + 'a,
    ) -> Self {
        CollapsibleTree {
            nodes,
            spacing: 0.0,
            indent: 14.0,
            filter: None,
            render_node: Box::new(render_node),
        }
    }

    /// Set vertical spacing between rows (default `0.0`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = px;
        self
    }

    /// Set horizontal indent per depth level in pixels (default `14.0`).
    pub fn indent(mut self, px: f32) -> Self {
        self.indent = px;
        self
    }

    /// Only show nodes whose data satisfies `f`.  Ancestors of matching
    /// descendants are kept visible so the tree structure is preserved.
    pub fn filter(mut self, f: impl Fn(&T) -> bool + 'a) -> Self {
        self.filter = Some(Box::new(f));
        self
    }

    /// Render the tree into an [`Element`].
    pub fn view(&self) -> Element<'a, Message, Theme, Renderer> {
        render_nodes(
            self.nodes,
            0,
            self.indent,
            self.spacing,
            self.filter.as_deref(),
            &self.render_node,
        )
        .unwrap_or_else(|| column![].into())
    }
}

// ── Internal rendering ───────────────────────────────────────────────

/// Recursively render visible nodes. Returns `None` when no node in this
/// subtree matches the filter.
#[allow(clippy::needless_range_loop)]
fn render_nodes<'a, T, Message: 'a, Theme: 'a + iced::widget::container::Catalog, Renderer: 'a + iced::advanced::Renderer>(
    nodes: &'a [TreeNode<T>],
    depth: usize,
    indent: f32,
    spacing: f32,
    filter: Option<&dyn Fn(&T) -> bool>,
    render: &impl Fn(RenderContext<'a, T>) -> Element<'a, Message, Theme, Renderer>,
) -> Option<Element<'a, Message, Theme, Renderer>> {
    let mut col: Column<'a, Message, Theme, Renderer> = column![].spacing(spacing);
    let mut count = 0usize;

    for node in nodes {
        let self_matches = filter.is_none_or(|f| f(&node.data));
        let child_content =
            render_nodes(&node.children, depth + 1, indent, spacing, filter, render);
        let child_visible = child_content.is_some();

        if !self_matches && !child_visible {
            continue;
        }
        count += 1;

        // Wrap the caller's content in a padded container for indentation.
        let left_pad = depth as f32 * indent;
        let ctx = RenderContext {
            data: &node.data,
            depth,
            expanded: node.expanded,
            has_children: !node.children.is_empty(),
        };
        let content = render(ctx);
        let padded = iced::widget::container(content)
            .padding(iced::Padding {
                top: 1.0,
                right: 0.0,
                bottom: 1.0,
                left: left_pad,
            })
            .width(iced::Length::Fill);
        col = col.push(padded);

        // Render children if expanded.
        if node.expanded && !node.children.is_empty() {
            if let Some(children) = child_content {
                col = col.push(children);
            }
        }
    }

    if count == 0 {
        None
    } else {
        Some(col.into())
    }
}
