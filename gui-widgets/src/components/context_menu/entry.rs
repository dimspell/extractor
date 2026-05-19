/// A context menu entry.
#[derive(Debug, Clone)]
pub enum Entry<Message> {
    Item {
        label: String,
        icon: Option<String>,
        action: Message,
    },
    Separator,
    Disabled {
        label: String,
        icon: Option<String>,
    },
}

impl<Message: Clone> Entry<Message> {
    pub fn item<S: Into<String>>(label: S, action: Message) -> Self {
        Entry::Item {
            label: label.into(),
            icon: None,
            action,
        }
    }

    #[allow(dead_code)]
    pub fn item_with_icon<S: Into<String>, I: Into<String>>(
        label: S,
        icon: I,
        action: Message,
    ) -> Self {
        Entry::Item {
            label: label.into(),
            icon: Some(icon.into()),
            action,
        }
    }

    pub fn separator() -> Self {
        Entry::Separator
    }

    pub fn disabled<S: Into<String>>(label: S) -> Self {
        Entry::Disabled {
            label: label.into(),
            icon: None,
        }
    }

    #[allow(dead_code)]
    pub fn disabled_with_icon<S: Into<String>, I: Into<String>>(label: S, icon: I) -> Self {
        Entry::Disabled {
            label: label.into(),
            icon: Some(icon.into()),
        }
    }
}
