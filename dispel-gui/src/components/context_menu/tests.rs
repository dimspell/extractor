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

#[test]
fn test_context_menu_tag() {
    let entries: Vec<Entry<TestMessage>> = vec![];
    let cm = ContextMenu::new(button("Test"), entries);
    assert_eq!(cm.tag(), tree::Tag::of::<State>());
}
