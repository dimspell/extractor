//! Headless tests for the toast manager's overlay and timing behavior.
//!
//! The auto-dismiss clock is driven by `Event::Window(RedrawRequested(..))`
//! (see `Overlay::update`), so these tests advance it explicitly instead of
//! sleeping. `ICED_TEST_BACKEND=software` is required by the iced test runner.

use super::*;
use iced_test::simulator;

fn redraw() -> Event {
    Event::Window(iced::window::Event::RedrawRequested(Instant::now()))
}

#[test]
fn test_toast_status_constructors() {
    assert_eq!(Toast::success("t", "b").status, Status::Success);
    assert_eq!(Toast::error("t", "b").status, Status::Danger);
    assert_eq!(Toast::info("t", "b").status, Status::Primary);
    assert_eq!(Toast::warning("t", "b").status, Status::Warning);
    assert_eq!(DEFAULT_TIMEOUT, 4);
}

#[test]
fn test_toast_is_rendered_as_overlay() {
    let toasts = vec![Toast::info("Saved", "file.snf")];
    let mut ui = simulator(Manager::new(text("base"), &toasts, |_| {
        "closed".to_string()
    }));

    ui.find("Saved").expect("toast title should be visible");
    ui.find("file.snf").expect("toast body should be visible");
}

#[test]
fn test_toast_auto_dismiss_publishes_on_close() {
    let toasts = vec![Toast::info("Saved", "file.snf")];
    // Timeout 0 means the first redraw already finds the deadline elapsed.
    let mut ui =
        simulator(Manager::new(text("base"), &toasts, |i| format!("close-{i}")).timeout(0));
    let _ = ui.simulate([redraw()]);

    let messages: Vec<String> = ui.into_messages().collect();
    assert_eq!(
        messages,
        vec!["close-0".to_string()],
        "expired toast must publish its on_close message"
    );
}

#[test]
fn test_toast_long_timeout_does_not_dismiss_on_redraw() {
    let toasts = vec![Toast::info("Saved", "file.snf")];
    let mut ui =
        simulator(Manager::new(text("base"), &toasts, |i| format!("close-{i}")).timeout(60));
    let _ = ui.simulate([redraw(), redraw()]);

    let messages: Vec<String> = ui.into_messages().collect();
    assert!(
        messages.is_empty(),
        "a toast with a long timeout must not dismiss on an early redraw"
    );
}

#[test]
fn test_toast_interaction_pauses_auto_dismiss() {
    let toasts = vec![Toast::info("Saved", "file.snf")];
    let close_icon = icon_char(Icon::X).to_string();
    let mut ui =
        simulator(Manager::new(text("base"), &toasts, |i| format!("close-{i}")).timeout(0));

    // Pressing the close button produces a local message; the overlay clears
    // that toast's deadline so the auto-dismiss path must not fire again.
    ui.click(close_icon.as_str())
        .expect("toast close button should be clickable");
    let _ = ui.simulate([redraw()]);

    let messages: Vec<String> = ui.into_messages().collect();
    assert_eq!(
        messages,
        vec!["close-0".to_string()],
        "interaction should publish once (the button) and suppress auto-dismiss"
    );
}
