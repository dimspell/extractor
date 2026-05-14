use iced::advanced::widget;
use iced::Point;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Status {
    #[default]
    Closed,
    Open {
        position: Point,
    },
}

impl Status {
    #[allow(dead_code)]
    pub fn position(self) -> Option<Point> {
        match self {
            Status::Closed => None,
            Status::Open { position } => Some(position),
        }
    }
}

pub struct State {
    pub(crate) status: Status,
    pub(crate) menu_tree: widget::Tree,
    pub(crate) hovered_idx: Option<usize>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            status: Status::Closed,
            menu_tree: widget::Tree::empty(),
            hovered_idx: None,
        }
    }
}
