#[macro_use]
pub mod macros;
pub mod editor;
pub mod ext;
pub mod startpage;
pub mod system;
pub mod viewer;
pub mod workspace;

pub use editor::EditorMessage;
pub use ext::{EditorMessageExt, MessageExt, WeaponEditorMessageExt};
pub use startpage::StartPageMessage;
pub use system::SystemMessage;
pub use viewer::ViewerMessage;
pub use workspace::WorkspaceMessage;

pub use crate::components::file_tree::FileTreeMessage;
use workspace::WorkspaceMessage as InternalWorkspaceMessage;

#[derive(Debug, Clone)]
pub enum Message {
    Workspace(InternalWorkspaceMessage),
    Editor(EditorMessage),
    FileTree(FileTreeMessage),
    Viewer(ViewerMessage),
    System(SystemMessage),
    StartPage(StartPageMessage),
}
