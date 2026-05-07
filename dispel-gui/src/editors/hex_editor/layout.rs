//! Structure-overlay registry.
//!
//! A [`BinaryLayout`] decomposes a byte buffer into named, typed [`FieldSpan`]s
//! that the matrix can color and the inspector can label. v1 ships an empty
//! registry — the trait + types let the rest of the editor reach for layouts
//! without knowing if any are wired.
//!
//! Phase 6b will derive `BinaryLayout` impls automatically from the
//! `#[extractor(...)]` byte-offset/size/type attributes already present on
//! `dispel_core::references::*` records.

use std::collections::HashMap;
use std::ops::Range;
use std::sync::OnceLock;

/// One named span inside a binary file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSpan {
    pub range: Range<u64>,
    pub name: &'static str,
    pub ty: &'static str,
}

/// Decompose a buffer's bytes into [`FieldSpan`]s. Implementations should
/// be cheap to call (the matrix may invoke them per file open).
pub trait BinaryLayout: Send + Sync {
    fn layout(&self, bytes: &[u8]) -> Vec<FieldSpan>;
}

/// Lookup table keyed by lower-cased extension (without leading dot).
pub struct LayoutRegistry {
    by_ext: HashMap<&'static str, &'static dyn BinaryLayout>,
}

impl LayoutRegistry {
    pub fn empty() -> Self {
        Self {
            by_ext: HashMap::new(),
        }
    }

    pub fn get(&self, ext: &str) -> Option<&'static dyn BinaryLayout> {
        self.by_ext.get(ext.to_ascii_lowercase().as_str()).copied()
    }

    pub fn is_empty(&self) -> bool {
        self.by_ext.is_empty()
    }
}

/// Process-wide registry. Empty in v1.
pub fn registry() -> &'static LayoutRegistry {
    static R: OnceLock<LayoutRegistry> = OnceLock::new();
    R.get_or_init(LayoutRegistry::empty)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubLayout;
    impl BinaryLayout for StubLayout {
        fn layout(&self, _bytes: &[u8]) -> Vec<FieldSpan> {
            vec![FieldSpan {
                range: 0..4,
                name: "header",
                ty: "u32",
            }]
        }
    }

    #[test]
    fn empty_registry_returns_none() {
        let r = LayoutRegistry::empty();
        assert!(r.get("db").is_none());
        assert!(r.is_empty());
    }

    #[test]
    fn stub_layout_returns_named_spans() {
        let l = StubLayout;
        let spans = l.layout(&[0; 8]);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].name, "header");
        assert_eq!(spans[0].range, 0..4);
    }

    #[test]
    fn process_registry_is_empty_by_default() {
        assert!(registry().is_empty());
    }
}
