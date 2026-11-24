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

fn entry(id: u64, title: &str, body: &str) -> (ToastId, String, String, Status) {
    (ToastId::new(id), title.into(), body.into(), Status::Primary)
}

fn old_timer(id: u64, title: &str, body: &str) -> ToastTimer {
    ToastTimer {
        id: ToastId::new(id),
        title: title.into(),
        body: body.into(),
        status: Status::Primary,
        started: Instant::now() - Duration::from_secs(100),
    }
}

#[test]
fn test_remove_middle_toast_keeps_other_timers() {
    let entries = [entry(1, "A", "a"), entry(2, "B", "b"), entry(3, "C", "c")];
    let old = Instant::now() - Duration::from_secs(100);
    let mut timers: ToastTimers = entries
        .iter()
        .map(|(id, title, body, status)| ToastTimer {
            id: id.clone(),
            title: title.clone(),
            body: body.clone(),
            status: *status,
            started: old,
        })
        .collect();

    sync_timers(&mut timers, &[entries[0].clone(), entries[2].clone()]);

    assert_eq!(
        timers.iter().map(|t| t.id.0).collect::<Vec<_>>(),
        vec![1, 3],
        "removing the middle toast must keep the surviving timers"
    );
    assert!(timers.iter().all(|t| t.started == old));
}

#[test]
fn test_replace_toast_content_resets_its_timer() {
    let old = old_timer(7, "A", "old body");
    let started_before = old.started;
    let mut timers = vec![old];
    sync_timers(&mut timers, &[entry(7, "A", "new body")]);
    assert_eq!(timers.len(), 1, "replacing content must not add a toast");
    assert!(
        timers[0].started > started_before,
        "replacing content must restart the countdown"
    );
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
