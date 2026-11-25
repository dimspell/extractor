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

/// Selected labels of every select box in `element`, in render order.
/// Selects paint their labels directly instead of exposing them as text
/// candidates, so read them from the accessibility tree where each select
/// publishes its current value.
#[cfg(feature = "iced_test")]
pub(crate) fn select_values<M: 'static>(element: iced::Element<'_, M>) -> Vec<String> {
    use iced_test::core::accessibility::accesskit;

    let mut ui = iced_test::simulator(element);
    ui.accessibility_tree()
        .nodes
        .into_iter()
        .filter(|(_, node)| node.role() == accesskit::Role::ComboBox)
        .filter_map(|(_, node)| node.value().map(str::to_owned))
        .collect()
}

/// Check the value rendered by a select box.
#[cfg(feature = "iced_test")]
pub(crate) fn select_shows<M: 'static>(element: iced::Element<'_, M>, value: &str) -> bool {
    select_values(element).iter().any(|v| v == value)
}

/// Focus the `nth` select box and activate its highlighted option, returning
/// the messages produced by that interaction.
#[cfg(feature = "iced_test")]
pub(crate) fn activate_select_option<M: 'static>(
    element: iced::Element<'_, M>,
    nth: usize,
) -> Vec<M> {
    use iced_test::core::accessibility::accesskit;
    use iced_test::instruction::Key;

    let mut ui = iced_test::simulator(element);
    let tree = ui.accessibility_tree();
    let targets: Vec<_> = tree
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == accesskit::Role::ComboBox)
        .map(|(id, _)| *id)
        .collect();
    let target = targets[nth];
    ui.accessibility_action(&accesskit::ActionRequest {
        action: accesskit::Action::Focus,
        target_tree: tree.tree_id,
        target_node: target,
        data: None,
    });
    ui.tap_key(Key::Enter);
    ui.tap_key(Key::Enter);
    ui.into_messages().collect()
}
