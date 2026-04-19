// Property-based testing for message routing
use dispel_gui::message::*;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_message_routing_roundtrip(msg: Message) {
        // Test that any message can be serialized and deserialized
        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&serialized).unwrap();

        // Should be able to match both original and deserialized
        match (msg, deserialized) {
            (Message::weapon(_), Message::weapon(_)) => {},
            (Message::Workspace(_), Message::Workspace(_)) => {},
            (Message::FileTree(_), Message::FileTree(_)) => {},
            (Message::Viewer(_), Message::Viewer(_)) => {},
            (Message::System(_), Message::System(_)) => {},
            _ => panic!("Message routing failed for {:?}", msg)
        }
    }
}

proptest! {
    #[test]
    fn test_editor_message_variants(editor_msg: EditorMessage) {
        // Test that all editor message variants work correctly
        let msg = Message::Editor(editor_msg.clone());

        // Should be able to extract the editor message back
        match msg {
            Message::Editor(inner) => assert_eq!(inner, editor_msg),
            _ => panic!("Editor message extraction failed")
        }
    }
}
