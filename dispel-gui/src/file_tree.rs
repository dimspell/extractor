use std::path::{Path, PathBuf};

use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Fill, Font, Length};

use crate::style;

/// A node in the file tree.
#[derive(Debug, Clone)]
pub enum TreeNode {
    Dir {
        path: PathBuf,
        name: String,
        expanded: bool,
        children: Vec<TreeNode>,
    },
    File {
        path: PathBuf,
        name: String,
        icon: &'static str,
    },
}

/// Messages from the file tree.
#[derive(Debug, Clone)]
pub enum FileTreeMessage {
    ToggleDir(PathBuf),
    OpenFile(PathBuf),
    Search(String),
}

/// File tree widget state.
#[derive(Debug, Clone, Default)]
pub struct FileTree {
    pub root: Option<TreeNode>,
    pub search_query: String,
}

impl FileTree {
    /// Scan a directory and build the tree.
    pub fn scan(path: &Path) -> Self {
        let root = scan_dir(path, 0);
        Self {
            root,
            search_query: String::new(),
        }
    }

    /// Toggle a directory's expanded state.
    pub fn toggle(&mut self, path: &Path) {
        if let Some(ref mut root) = self.root {
            toggle_node(root, path);
        }
    }

    /// Render the tree.
    pub fn view(&self) -> Element<'_, FileTreeMessage> {
        let search_bar = text_input("🔍 Filter files...", &self.search_query)
            .on_input(FileTreeMessage::Search)
            .padding(6)
            .size(12);

        let content = match &self.root {
            Some(node) => render_node(node, &self.search_query),
            None => column![text("No game path set").size(12).style(style::subtle_text)].into(),
        };

        column![
            container(search_bar).padding(8),
            scrollable(content).height(Length::Fill),
        ]
        .spacing(0)
        .height(Fill)
        .into()
    }
}

fn scan_dir(path: &Path, depth: usize) -> Option<TreeNode> {
    if depth > 3 {
        return None;
    }

    let name = path.file_name()?.to_string_lossy().to_string();

    if path.is_dir() {
        Some(TreeNode::Dir {
            path: path.to_path_buf(),
            name,
            expanded: depth == 0,
            children: if depth == 0 {
                scan_children(path)
            } else {
                Vec::new()
            },
        })
    } else {
        let icon = file_icon(path);
        Some(TreeNode::File {
            path: path.to_path_buf(),
            name,
            icon,
        })
    }
}

fn scan_children(path: &Path) -> Vec<TreeNode> {
    let mut children = Vec::new();
    if let Ok(entries) = std::fs::read_dir(path) {
        let mut dirs: Vec<_> = Vec::new();
        let mut files: Vec<_> = Vec::new();

        for entry in entries.flatten() {
            let p = entry.path();
            if let Some(node) = scan_dir(&p, 1) {
                match &node {
                    TreeNode::Dir { .. } => dirs.push(node),
                    TreeNode::File { .. } => files.push(node),
                }
            }
        }

        dirs.sort_by_key(|n| match n {
            TreeNode::Dir { name, .. } => name.to_lowercase(),
            TreeNode::File { name, .. } => name.to_lowercase(),
        });
        files.sort_by_key(|n| match n {
            TreeNode::Dir { name, .. } => name.to_lowercase(),
            TreeNode::File { name, .. } => name.to_lowercase(),
        });

        children.extend(dirs);
        children.extend(files);
    }
    children
}

fn toggle_node(node: &mut TreeNode, path: &Path) {
    match node {
        TreeNode::Dir {
            path: dir_path,
            expanded,
            children,
            ..
        } => {
            if dir_path == path {
                *expanded = !*expanded;
                if *expanded && children.is_empty() {
                    *children = scan_children(dir_path);
                }
            } else {
                for child in children {
                    toggle_node(child, path);
                }
            }
        }
        TreeNode::File { .. } => {}
    }
}

fn render_node<'a>(node: &'a TreeNode, query: &'a str) -> Element<'a, FileTreeMessage> {
    match node {
        TreeNode::Dir {
            path,
            name,
            expanded,
            children,
        } => {
            let icon = if *expanded { "📂" } else { "📁" };
            let header: Element<'a, FileTreeMessage> =
                button(row![text(icon).size(12), text(name).size(12)].spacing(4))
                    .on_press(FileTreeMessage::ToggleDir(path.clone()))
                    .width(Fill)
                    .style(style::chip)
                    .into();

            let mut content: iced::widget::Column<'a, FileTreeMessage> =
                iced::widget::Column::new().push(header);
            if *expanded {
                for child in children {
                    content = content.push(render_node(child, query));
                }
            }
            Element::from(container(content).padding([2, 0]))
        }
        TreeNode::File {
            path, name, icon, ..
        } => {
            if !query.is_empty() && !name.to_lowercase().contains(&query.to_lowercase()) {
                return Element::from(container(text(" ")).width(Fill));
            }
            Element::from(
                button(
                    row![
                        text(*icon).size(10),
                        text(name).size(11).font(Font::MONOSPACE),
                    ]
                    .spacing(4),
                )
                .on_press(FileTreeMessage::OpenFile(path.clone()))
                .width(Fill)
                .style(style::chip),
            )
        }
    }
}

fn file_icon(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("db") => "🗃️",
        Some("ini") => "📄",
        Some("ref") => "📋",
        Some("scr") => "📜",
        Some("dlg") => "💬",
        Some("pgp") => "📝",
        Some("map") => "🗺️",
        Some("gtl") | Some("btl") => "🖼️",
        Some("spr") => "🎨",
        Some("snf") => "🔊",
        _ => "📎",
    }
}
