//! Mapping from relative game-file paths to [`RecordPatcher`] handlers.
//!
//! Two registration shapes:
//!
//! * **Exact filename** (case-insensitive on the last path component).
//!   `MiscItem.db` matches `CharacterInGame/MiscItem.db` regardless of how
//!   the user has nested it. Use [`Self::register`].
//! * **Extension + stem-prefix pattern.** Used by catalogs whose filename
//!   varies per map, e.g. `Extdun01.ref`, `Extfld01.ref`, ... — registered
//!   once with `extension = "ref"`, `stem_prefix = "ext"`. Use
//!   [`Self::register_pattern`].
//!
//! Lookup tries the exact-filename map first (O(1)), then falls back to the
//! pattern list (O(N) but N is small).

use std::collections::HashMap;
use std::sync::Arc;

use super::patcher::RecordPatcher;

/// Indexed collection of [`RecordPatcher`]s.
#[derive(Default, Clone)]
pub struct PatcherRegistry {
    by_filename: HashMap<String, Arc<dyn RecordPatcher>>,
    /// `(lowercase_extension, lowercase_stem_prefix, patcher)`.
    patterns: Vec<(String, String, Arc<dyn RecordPatcher>)>,
}

impl PatcherRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Pre-populated registry containing every [`RecordPatcher`] dispel-core
    /// ships out of the box. Each entry's filename / pattern comes from the
    /// `#[patcher(...)]` constants on the generated patcher, keeping the
    /// registration site single-sourced with the derive.
    pub fn with_defaults() -> Self {
        use super::patchers::{
            EditItemPatcher, EventItemPatcher, ExtraRefPatcher, HealItemPatcher, MiscItemPatcher,
            PartyLevelDbPatcher,
        };
        let mut r = Self::new();
        r.register(MiscItemPatcher::FILENAME, Arc::new(MiscItemPatcher));
        r.register(HealItemPatcher::FILENAME, Arc::new(HealItemPatcher));
        r.register(EditItemPatcher::FILENAME, Arc::new(EditItemPatcher));
        r.register(EventItemPatcher::FILENAME, Arc::new(EventItemPatcher));
        r.register(PartyLevelDbPatcher::FILENAME, Arc::new(PartyLevelDbPatcher));
        r.register_pattern(
            ExtraRefPatcher::EXTENSION,
            ExtraRefPatcher::STEM_PREFIX,
            Arc::new(ExtraRefPatcher),
        );
        r
    }

    /// Register a patcher for files whose name (the last path component)
    /// equals `filename`, case-insensitively.
    pub fn register(&mut self, filename: &str, patcher: Arc<dyn RecordPatcher>) {
        self.by_filename
            .insert(filename.to_ascii_lowercase(), patcher);
    }

    /// Register a patcher matching every file whose extension equals
    /// `extension` and whose filename stem starts with `stem_prefix`,
    /// both compared case-insensitively.
    pub fn register_pattern(
        &mut self,
        extension: &str,
        stem_prefix: &str,
        patcher: Arc<dyn RecordPatcher>,
    ) {
        self.patterns.push((
            extension.to_ascii_lowercase(),
            stem_prefix.to_ascii_lowercase(),
            patcher,
        ));
    }

    /// Look up the handler for a relative path. Tries the exact-filename
    /// map first, then walks the pattern list.
    pub fn lookup(&self, relative_path: &str) -> Option<Arc<dyn RecordPatcher>> {
        let path = std::path::Path::new(relative_path);
        let filename = path.file_name().and_then(|s| s.to_str())?.to_ascii_lowercase();
        if let Some(p) = self.by_filename.get(&filename) {
            return Some(p.clone());
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())?
            .to_ascii_lowercase();
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        for (pat_ext, pat_prefix, patcher) in &self.patterns {
            if pat_ext == &ext && stem.starts_with(pat_prefix.as_str()) {
                return Some(patcher.clone());
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.by_filename.len() + self.patterns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_filename.is_empty() && self.patterns.is_empty()
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
    fn pattern_matches_by_extension_and_stem_prefix() {
        let mut r = PatcherRegistry::new();
        r.register_pattern("ref", "ext", Arc::new(Stub("extra")));

        assert!(r.lookup("NpcInGame/Extdun01.ref").is_some());
        assert!(r.lookup("NpcInGame/extfld02.ref").is_some());
        assert!(r.lookup("NpcInGame/EXTBOSS.REF").is_some());
        // Wrong extension — no match.
        assert!(r.lookup("NpcInGame/Extdun01.db").is_none());
        // Wrong prefix — no match.
        assert!(r.lookup("NpcInGame/Npcdun01.ref").is_none());
    }

    #[test]
    fn exact_match_wins_over_pattern() {
        let mut r = PatcherRegistry::new();
        r.register_pattern("ref", "ext", Arc::new(Stub("pattern")));
        r.register("Extdun01.ref", Arc::new(Stub("exact")));
        let p = r.lookup("NpcInGame/Extdun01.ref").unwrap();
        assert_eq!(p.name(), "exact");
    }

    #[test]
    fn defaults_include_all_generated_patchers() {
        let r = PatcherRegistry::with_defaults();
        for path in [
            "CharacterInGame/MiscItem.db",
            "CharacterInGame/HealItem.db",
            "CharacterInGame/EditItem.db",
            "CharacterInGame/EventItem.db",
            "NpcInGame/Extdun01.ref",
            "NpcInGame/Extfld02.ref",
            "NpcInGame/PrtLevel.db",
        ] {
            assert!(r.lookup(path).is_some(), "registry missing handler for {path}");
        }
    }
}
