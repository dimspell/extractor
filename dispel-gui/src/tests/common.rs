//! Common test helpers shared across test modules.

use crate::app::App;
use crate::state::RecordingSession;
use crate::workspace::Workspace;

/// Create an App with an active recording session set up for testing.
pub(crate) fn app_with_recording() -> App {
    let mut app = App::test_new(Workspace::new());
    app.state.recording = Some(RecordingSession {
        workspace_root: std::env::temp_dir(),
        mod_slug: "test_mod".to_string(),
        mod_name: "Test Mod".to_string(),
        ..Default::default()
    });
    app
}

/// Create an App without a recording session (plain test app).
pub(crate) fn app_without_recording() -> App {
    App::test_new(Workspace::new())
}

/// Check the value rendered by a select box. Selects paint their selected
/// label directly instead of exposing it as a text candidate, so read it from
/// the accessibility tree where it is published as the widget's value.
#[cfg(feature = "iced_test")]
pub(crate) fn select_shows<M: 'static>(element: iced::Element<'_, M>, value: &str) -> bool {
    let mut ui = iced_test::simulator(element);
    ui.accessibility_tree()
        .nodes
        .iter()
        .any(|(_, node)| node.value() == Some(value))
}
