//! Mapping from relative game-file paths to [`RecordPatcher`] handlers.
//!
//! Lookup is by **case-insensitive filename suffix**: registering
//! `"MiscItem.db"` matches `CharacterInGame/MiscItem.db` regardless of the
//! directory layout the user ships. This mirrors how the GUI's
//! `EditorType::from_path` already routes catalog files.

use std::collections::HashMap;
use std::sync::Arc;

use super::patcher::RecordPatcher;

/// Indexed collection of [`RecordPatcher`]s keyed by filename suffix.
#[derive(Default, Clone)]
pub struct PatcherRegistry {
    by_filename: HashMap<String, Arc<dyn RecordPatcher>>,
}

impl PatcherRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Pre-populated registry containing every [`RecordPatcher`] dispel-core
    /// ships out of the box. Phase 2 only includes `MiscItem.db`; later
    /// phases will expand this list.
    pub fn with_defaults() -> Self {
        let mut r = Self::new();
        r.register("MiscItem.db", Arc::new(super::patchers::MiscItemPatcher));
        r
    }

    /// Register a patcher for files whose name (the last path component)
    /// equals `filename`, case-insensitively.
    pub fn register(&mut self, filename: &str, patcher: Arc<dyn RecordPatcher>) {
        self.by_filename
            .insert(filename.to_ascii_lowercase(), patcher);
    }

    /// Look up the handler for a relative path, by its filename component.
    pub fn lookup(&self, relative_path: &str) -> Option<Arc<dyn RecordPatcher>> {
        let filename = std::path::Path::new(relative_path)
            .file_name()
            .and_then(|s| s.to_str())?
            .to_ascii_lowercase();
        self.by_filename.get(&filename).cloned()
    }

    pub fn len(&self) -> usize {
        self.by_filename.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_filename.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modding::error::Result;
    use crate::modding::value::Value;

    struct Stub(&'static str);
    impl RecordPatcher for Stub {
        fn name(&self) -> &'static str {
            self.0
        }
        fn apply_field(&self, _: &[u8], _: u32, _: &str, _: &Value) -> Result<Vec<u8>> {
            Ok(vec![])
        }
    }

    #[test]
    fn lookup_matches_filename_case_insensitively() {
        let mut r = PatcherRegistry::new();
        r.register("MiscItem.db", Arc::new(Stub("misc")));

        assert!(r.lookup("CharacterInGame/MiscItem.db").is_some());
        assert!(r.lookup("character/miscitem.db").is_some());
        assert!(r.lookup("MISCITEM.DB").is_some());
        assert!(r.lookup("Other.db").is_none());
    }

    #[test]
    fn defaults_include_misc_item() {
        let r = PatcherRegistry::with_defaults();
        assert!(r.lookup("CharacterInGame/MiscItem.db").is_some());
    }
}
