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
        use super::patchers::*;
        let mut r = Self::new();

        // Binary catalogs (.db) — derived patchers.
        r.register(MiscItemPatcher::FILENAME, Arc::new(MiscItemPatcher));
        r.register(HealItemPatcher::FILENAME, Arc::new(HealItemPatcher));
        r.register(EditItemPatcher::FILENAME, Arc::new(EditItemPatcher));
        r.register(EventItemPatcher::FILENAME, Arc::new(EventItemPatcher));
        r.register(MonsterPatcher::FILENAME, Arc::new(MonsterPatcher));
        r.register(WeaponItemPatcher::FILENAME, Arc::new(WeaponItemPatcher));
        r.register(MagicSpellPatcher::FILENAME, Arc::new(MagicSpellPatcher));
        // Magic.db has a sibling MulMagic.db with the same record format —
        // route both filenames to the same patcher.
        r.register("MulMagic.db", Arc::new(MagicSpellPatcher));
        r.register(PartyIniNpcPatcher::FILENAME, Arc::new(PartyIniNpcPatcher));

        // Binary catalogs (.db) — hand-written.
        r.register(PartyLevelDbPatcher::FILENAME, Arc::new(PartyLevelDbPatcher));
        r.register(StorePatcher::FILENAME, Arc::new(StorePatcher));
        r.register(ChDataPatcher::FILENAME, Arc::new(ChDataPatcher));

        // Binary refs that vary per map.
        r.register_pattern(
            ExtraRefPatcher::EXTENSION,
            ExtraRefPatcher::STEM_PREFIX,
            Arc::new(ExtraRefPatcher),
        );
        r.register_pattern(
            MonsterRefPatcher::EXTENSION,
            MonsterRefPatcher::STEM_PREFIX,
            Arc::new(MonsterRefPatcher),
        );
        r.register_pattern(
            NPCPatcher::EXTENSION,
            NPCPatcher::STEM_PREFIX,
            Arc::new(NPCPatcher),
        );

        // Text catalogs (.ini / .ref) — derived text patchers.
        r.register(MapPatcher::FILENAME, Arc::new(MapPatcher));
        r.register(MapIniPatcher::FILENAME, Arc::new(MapIniPatcher));
        r.register(MonsterIniPatcher::FILENAME, Arc::new(MonsterIniPatcher));
        r.register(NpcIniPatcher::FILENAME, Arc::new(NpcIniPatcher));
        r.register(EventPatcher::FILENAME, Arc::new(EventPatcher));
        r.register(ExtraPatcher::FILENAME, Arc::new(ExtraPatcher));
        r.register(EventNpcRefPatcher::FILENAME, Arc::new(EventNpcRefPatcher));
        r.register(MessagePatcher::FILENAME, Arc::new(MessagePatcher));
        r.register(WaveIniPatcher::FILENAME, Arc::new(WaveIniPatcher));
        r.register(PartyRefPatcher::FILENAME, Arc::new(PartyRefPatcher));

        // Text catalogs — hand-written.
        r.register(DrawItemPatcher::FILENAME, Arc::new(DrawItemPatcher));
        r.register(QuestPatcher::FILENAME, Arc::new(QuestPatcher));

        // Per-file event scripts: every Event*.scr matches. Exact-filename
        // matches for Quest.scr and Message.scr win over this via the
        // by_filename map, so they aren't shadowed.
        r.register_pattern(
            EventScriptPatcher::EXTENSION,
            EventScriptPatcher::STEM_PREFIX,
            Arc::new(EventScriptPatcher),
        );

        // Per-file dialogue formats: every *.dlg / *.pgp matches.
        r.register_pattern(
            DialogueScriptPatcher::EXTENSION,
            DialogueScriptPatcher::STEM_PREFIX,
            Arc::new(DialogueScriptPatcher),
        );
        r.register_pattern(
            DialogueParagraphPatcher::EXTENSION,
            DialogueParagraphPatcher::STEM_PREFIX,
            Arc::new(DialogueParagraphPatcher),
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
    fn quest_and_message_scr_resolve_to_their_exact_patchers_not_event() {
        let r = PatcherRegistry::with_defaults();
        // Both files share the .scr extension with Event*.scr but must hit
        // their dedicated handlers via the exact-filename map.
        assert_eq!(
            r.lookup("ExtraInGame/Quest.scr").unwrap().name(),
            "Quest"
        );
        assert_eq!(
            r.lookup("ExtraInGame/Message.scr").unwrap().name(),
            "Message"
        );
        assert_eq!(
            r.lookup("ExtraInGame/Eventcat1.scr").unwrap().name(),
            "EventScript"
        );
    }

    #[test]
    fn defaults_include_all_generated_patchers() {
        let r = PatcherRegistry::with_defaults();
        for path in [
            // Binary .db (derived)
            "CharacterInGame/MiscItem.db",
            "CharacterInGame/HealItem.db",
            "CharacterInGame/EditItem.db",
            "CharacterInGame/EventItem.db",
            "MonsterInGame/Monster.db",
            "CharacterInGame/WeaponItem.db",
            "MagicInGame/Magic.db",
            "MagicInGame/MulMagic.db",
            "NpcInGame/PrtIni.db",
            // Binary .db (hand-written)
            "NpcInGame/PrtLevel.db",
            // Per-map binary refs (pattern-matched)
            "NpcInGame/Extdun01.ref",
            "NpcInGame/Extfld02.ref",
            "MonsterInGame/Mondun01.ref",
            "MonsterInGame/Monfld02.ref",
            "NpcInGame/Npccat1.ref",
            "NpcInGame/Npcdun02.ref",
            // Text catalogs (derived)
            "AllMap.ini",
            "Ref/Map.ini",
            "Monster.ini",
            "Npc.ini",
            "Event.ini",
            "Extra.ini",
            "NpcInGame/Eventnpc.ref",
            "Wave.ini",
            "Ref/PartyRef.ref",
            // Text/ad-hoc (hand-written)
            "Ref/DRAWITEM.ref",
            "ExtraInGame/Quest.scr",
            "ExtraInGame/Message.scr",
            "CharacterInGame/Store.db",
            "CharacterInGame/ChData.db",
            // Per-file dialogue (.dlg / .pgp) match every stem
            "NpcInGame/Dlgcat1.dlg",
            "NpcInGame/Dlgdun02.dlg",
            "NpcInGame/Pgpcat1.pgp",
            "NpcInGame/somefile.pgp",
            // Per-file event scripts match every Event*.scr
            "ExtraInGame/Eventcat1.scr",
            "ExtraInGame/EVENTDUN02.scr",
        ] {
            assert!(r.lookup(path).is_some(), "registry missing handler for {path}");
        }
    }
}
