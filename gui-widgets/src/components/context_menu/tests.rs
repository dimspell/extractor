use super::*;
use iced::widget::button;

type TestMessage = String;

#[test]
fn test_entry_item() {
    let entry: Entry<TestMessage> = Entry::item("Test", "action".into());
    assert!(matches!(entry, Entry::Item { label, .. } if label == "Test"));
}

#[test]
fn test_entry_separator() {
    let entry: Entry<TestMessage> = Entry::separator();
    assert!(matches!(entry, Entry::Separator));
}

#[test]
fn test_entry_disabled() {
    let entry: Entry<TestMessage> = Entry::disabled("Unavailable");
    assert!(matches!(entry, Entry::Disabled { label, .. } if label == "Unavailable"));
}

#[test]
fn test_status_default_closed() {
    let status = Status::default();
    assert!(matches!(status, Status::Closed));
}

#[test]
fn test_status_position() {
    let status = Status::Open {
        position: Point::new(100.0, 200.0),
    };
    assert_eq!(status.position(), Some(Point::new(100.0, 200.0)));
}

#[test]
fn test_context_menu_new() {
    let entries = vec![Entry::item("Option", "msg".into())];
    let cm = ContextMenu::new(button("Test"), entries);
    let _: Element<'static, TestMessage> = cm.into();
}

#[test]
fn test_context_menu_from_simple() {
    let entries = vec![("Option".to_string(), "msg".into())];
    let cm = ContextMenu::from_simple(button("Test"), entries);
    let _: Element<'static, TestMessage> = cm.into();
}

#[test]
fn test_context_menu_with_separator() {
    let entries = vec![
        Entry::item("Copy", "copy".into()),
        Entry::separator(),
        Entry::item("Paste", "paste".into()),
    ];
    let cm = ContextMenu::new(button("Test"), entries);
    let _: Element<'static, TestMessage> = cm.into();
}

#[test]
fn test_context_menu_with_disabled() {
    let entries = vec![
        Entry::item("Enabled", "enabled".into()),
        Entry::separator(),
        Entry::disabled("Not available"),
    ];
    let cm = ContextMenu::new(button("Test"), entries);
    let _: Element<'static, TestMessage> = cm.into();
}

#[test]
fn test_context_menu_empty_entries() {
    let entries: Vec<Entry<TestMessage>> = vec![];
    let cm = ContextMenu::new(button("Test"), entries);
    let _: Element<'static, TestMessage> = cm.into();
}

// ── Additional tests ──────────────────────────────────────────────────────────

#[test]
fn test_entry_item_with_icon() {
    let entry: Entry<TestMessage> = Entry::item_with_icon("Copy", "📋", "copy_msg".into());
    assert!(
        matches!(entry, Entry::Item { label, icon, .. } if label == "Copy" && icon.as_deref() == Some("📋"))
    );
}

#[test]
fn test_entry_disabled_with_icon() {
    let entry: Entry<TestMessage> = Entry::disabled_with_icon("Locked", "🔒");
    assert!(
        matches!(entry, Entry::Disabled { label, icon, .. } if label == "Locked" && icon.as_deref() == Some("🔒"))
    );
}

#[test]
fn test_status_default_from_enum() {
    assert_eq!(Status::Closed, Status::default());
}

#[test]
fn test_status_closed_position_returns_none() {
    assert_eq!(Status::Closed.position(), None);
}

#[test]
fn test_context_menu_offset() {
    // Verify offset builder method returns Self and converts to Element
    let cm = ContextMenu::new(button("test"), Vec::<Entry<TestMessage>>::new())
        .offset(Point::new(10.0, 20.0));
    let _: Element<'static, TestMessage> = cm.into();

    // Also verify via new() that .offset() returns Self (builder pattern)
    let cm2 = ContextMenu::new(button("test2"), Vec::<Entry<TestMessage>>::new());
    let _cm2_with_offset = cm2.offset(Point::new(5.0, 5.0));
}

#[test]
fn test_context_menu_default_offset() {
    let cm = ContextMenu::from_simple(button("test"), Vec::<(String, TestMessage)>::new());
    let _: Element<'static, TestMessage> = cm.into();
}

#[test]
fn test_entry_debug_clone() {
    let entry: Entry<TestMessage> = Entry::item("ClickMe", "action".into());
    let cloned = entry.clone();
    let debug = format!("{:?}", cloned);
    assert!(
        debug.contains("ClickMe"),
        "Debug output should contain the label: {}",
        debug
    );
}

#[test]
fn test_platform_force_custom_env_returns_none() {
    unsafe { std::env::set_var("FORCE_CUSTOM_CONTEXT_MENU", "1") };
    let entries: Vec<Entry<String>> = vec![Entry::item("Test", "action".into())];
    let result = super::platform::try_show_native_menu(&entries);
    assert!(
        result.is_none(),
        "FORCE_CUSTOM_CONTEXT_MENU should skip native menus"
    );
    unsafe { std::env::remove_var("FORCE_CUSTOM_CONTEXT_MENU") };
}

#[test]
fn test_native_result_clone_copy() {
    let selected = super::platform::NativeResult::Selected(5);
    let cancelled = super::platform::NativeResult::Cancelled;
    // Copy semantics — NativeResult derives Copy so the originals are not moved
    let selected_clone = selected;
    let cancelled_clone = cancelled;
    assert!(matches!(
        selected_clone,
        super::platform::NativeResult::Selected(5)
    ));
    assert!(matches!(
        cancelled_clone,
        super::platform::NativeResult::Cancelled
    ));
    // Verify originals are still usable (Copy)
    assert!(matches!(selected, super::platform::NativeResult::Selected(5)));
    assert!(matches!(cancelled, super::platform::NativeResult::Cancelled));
}

#[test]
fn test_platform_empty_entries_returns_none() {
    let entries: Vec<Entry<String>> = vec![];
    let result = super::platform::try_show_native_menu(&entries);
    assert!(result.is_none());
}
