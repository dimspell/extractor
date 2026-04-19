/// Messages from the tab bar.
#[derive(Debug, Clone, PartialEq)]
pub enum TabBarMessage {
    SelectTab(usize),
    CloseTab(usize),
    CloseOthers(usize),
    CloseAll,
    TogglePin(usize),
    CloseActiveTab,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════════════════
    // TabBarMessage Creation Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_select_tab_message() {
        let msg = TabBarMessage::SelectTab(0);
        assert!(matches!(msg, TabBarMessage::SelectTab(0)));
    }

    #[test]
    fn test_select_tab_message_various_indices() {
        let indices = [0, 1, 5, 100, usize::MAX];
        for idx in indices {
            let msg = TabBarMessage::SelectTab(idx);
            assert!(matches!(msg, TabBarMessage::SelectTab(i) if i == idx));
        }
    }

    #[test]
    fn test_close_tab_message() {
        let msg = TabBarMessage::CloseTab(0);
        assert!(matches!(msg, TabBarMessage::CloseTab(0)));
    }

    #[test]
    fn test_close_others_message() {
        let msg = TabBarMessage::CloseOthers(0);
        assert!(matches!(msg, TabBarMessage::CloseOthers(0)));
    }

    #[test]
    fn test_close_all_message() {
        let msg = TabBarMessage::CloseAll;
        assert!(matches!(msg, TabBarMessage::CloseAll));
    }

    #[test]
    fn test_toggle_pin_message() {
        let msg = TabBarMessage::TogglePin(0);
        assert!(matches!(msg, TabBarMessage::TogglePin(0)));
    }

    #[test]
    fn test_close_active_tab_message() {
        let msg = TabBarMessage::CloseActiveTab;
        assert!(matches!(msg, TabBarMessage::CloseActiveTab));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TabBarMessage Equality Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_select_tab_equality() {
        let msg1 = TabBarMessage::SelectTab(5);
        let msg2 = TabBarMessage::SelectTab(5);
        let msg3 = TabBarMessage::SelectTab(10);
        assert_eq!(msg1, msg2);
        assert_ne!(msg1, msg3);
    }

    #[test]
    fn test_close_tab_equality() {
        let msg1 = TabBarMessage::CloseTab(3);
        let msg2 = TabBarMessage::CloseTab(3);
        let msg3 = TabBarMessage::CloseTab(7);
        assert_eq!(msg1, msg2);
        assert_ne!(msg1, msg3);
    }

    #[test]
    fn test_close_others_equality() {
        let msg1 = TabBarMessage::CloseOthers(2);
        let msg2 = TabBarMessage::CloseOthers(2);
        let msg3 = TabBarMessage::CloseOthers(4);
        assert_eq!(msg1, msg2);
        assert_ne!(msg1, msg3);
    }

    #[test]
    fn test_toggle_pin_equality() {
        let msg1 = TabBarMessage::TogglePin(1);
        let msg2 = TabBarMessage::TogglePin(1);
        let msg3 = TabBarMessage::TogglePin(2);
        assert_eq!(msg1, msg2);
        assert_ne!(msg1, msg3);
    }

    #[test]
    fn test_close_all_is_unique() {
        let msg1 = TabBarMessage::CloseAll;
        let msg2 = TabBarMessage::CloseAll;
        assert_eq!(msg1, msg2);
    }

    #[test]
    fn test_close_active_tab_is_unique() {
        let msg1 = TabBarMessage::CloseActiveTab;
        let msg2 = TabBarMessage::CloseActiveTab;
        assert_eq!(msg1, msg2);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TabBarMessage Clone Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_message_clone() {
        let msg = TabBarMessage::SelectTab(42);
        let cloned = msg.clone();
        assert_eq!(msg, cloned);
    }

    #[test]
    fn test_all_messages_are_cloneable() {
        let messages = vec![
            TabBarMessage::SelectTab(0),
            TabBarMessage::CloseTab(1),
            TabBarMessage::CloseOthers(2),
            TabBarMessage::CloseAll,
            TabBarMessage::TogglePin(3),
            TabBarMessage::CloseActiveTab,
        ];
        for msg in messages {
            let cloned = msg.clone();
            assert_eq!(msg, cloned);
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TabBarMessage Debug Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_message_debug() {
        let msg = TabBarMessage::SelectTab(5);
        let debug = format!("{:?}", msg);
        assert!(debug.contains("SelectTab"));
        assert!(debug.contains("5"));
    }

    #[test]
    fn test_close_all_debug() {
        let msg = TabBarMessage::CloseAll;
        let debug = format!("{:?}", msg);
        assert!(debug.contains("CloseAll"));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TabBarMessage Pattern Matching Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_pattern_match_all_variants() {
        let messages = vec![
            TabBarMessage::SelectTab(0),
            TabBarMessage::CloseTab(0),
            TabBarMessage::CloseOthers(0),
            TabBarMessage::CloseAll,
            TabBarMessage::TogglePin(0),
            TabBarMessage::CloseActiveTab,
        ];
        for msg in messages {
            match &msg {
                TabBarMessage::SelectTab(_) => {}
                TabBarMessage::CloseTab(_) => {}
                TabBarMessage::CloseOthers(_) => {}
                TabBarMessage::CloseAll => {}
                TabBarMessage::TogglePin(_) => {}
                TabBarMessage::CloseActiveTab => {}
            }
        }
    }
}
