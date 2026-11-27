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

/// Focus the `nth` select box, move the highlight by `offset` entries, and
/// activate it. Returns the messages produced by the interactions.
#[cfg(feature = "iced_test")]
pub(crate) fn activate_select_option<M: 'static>(
    element: iced::Element<'_, M>,
    nth: usize,
    offset: isize,
) -> Vec<M> {
    use iced_test::core::accessibility::accesskit;
    use iced_test::instruction::Key;
    use iced_test::selector::{Candidate, Target};

    let mut ui = iced_test::simulator(element);
    let tree = ui.accessibility_tree();
    let targets: Vec<_> = tree
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == accesskit::Role::ComboBox)
        .map(|(id, node)| (*id, node.child_supports_action(accesskit::Action::Click)))
        .collect();
    let (target, is_combo_box) = targets[nth];
    if is_combo_box {
        // Click to open the dropdown. The widget clears its seeded selection
        // on focus; the extra Backspaces keep the simulated input's query
        // empty so arrow navigation starts from the full option list.
        let mut seen = 0;
        let mut text_len = 0;
        let mut select = |candidate: Candidate<'_>| match candidate {
            Candidate::TextInput { state, .. } if seen == nth => {
                seen += 1;
                text_len = state.text().chars().count();
                Some(Target::from(candidate))
            }
            Candidate::TextInput { .. } => {
                seen += 1;
                None
            }
            _ => None,
        };
        ui.click(&mut select).expect("combo box must be clickable");
        for _ in 0..text_len {
            ui.tap_key(Key::Backspace);
        }
    } else {
        ui.accessibility_action(&accesskit::ActionRequest {
            action: accesskit::Action::Focus,
            target_tree: tree.tree_id,
            target_node: target,
            data: None,
        });
        // A pick_list opens on Enter; a ComboBox is already open once clicked.
        ui.tap_key(Key::Enter);
    }

    let named = if offset < 0 {
        iced::keyboard::key::Named::ArrowUp
    } else {
        iced::keyboard::key::Named::ArrowDown
    };
    for _ in 0..offset.unsigned_abs() {
        ui.simulate([iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(named),
            modified_key: iced::keyboard::Key::Named(named),
            physical_key: iced::keyboard::key::Physical::Unidentified(
                iced::keyboard::key::NativeCode::Unidentified,
            ),
            location: iced::keyboard::Location::Standard,
            modifiers: iced::keyboard::Modifiers::empty(),
            text: None,
            repeat: false,
        })]);
    }

    ui.tap_key(Key::Enter);
    ui.into_messages().collect()
}

/// Focus the `nth` combo box, type a filter query, and activate the highlighted
/// option. Returns the messages produced by the interaction.
#[cfg(feature = "iced_test")]
pub(crate) fn filter_select<M: 'static>(
    element: iced::Element<'_, M>,
    nth: usize,
    query: &str,
) -> Vec<M> {
    use iced_test::instruction::Key;
    use iced_test::selector::{Candidate, Target};

    let mut ui = iced_test::simulator(element);
    let mut seen = 0;
    let mut text_len = 0;
    let mut select = |candidate: Candidate<'_>| match candidate {
        Candidate::TextInput { state, .. } if seen == nth => {
            seen += 1;
            text_len = state.text().chars().count();
            Some(Target::from(candidate))
        }
        Candidate::TextInput { .. } => {
            seen += 1;
            None
        }
        _ => None,
    };
    // Click to open the combo. The widget clears its seeded selection on
    // focus; the Backspaces keep the simulated query empty before typing so
    // filtering starts from every option.
    ui.click(&mut select).expect("combo box must be clickable");
    for _ in 0..text_len {
        ui.tap_key(Key::Backspace);
    }
    ui.typewrite(query);
    ui.tap_key(Key::Enter);
    ui.into_messages().collect()
}
