//! Per-field conflict resolution.
//!
//! When two enabled mods write the same `(file, record_id, field)` with
//! different values, the apply engine defaults to load-order resolution
//! (last writer wins). The user can override this on a per-key basis by
//! pinning the conflict to a specific source mod's value: that mod's
//! contribution wins regardless of where it sits in the load order.
//!
//! Pins are workspace-level state — they are *not* part of any mod's
//! exportable artifact. They live in `<workspace>/resolutions.json` and
//! survive enable/disable but are silently ignored when the pinned mod is
//! not currently enabled (the conflict lapses, so does the pin).
//!
//! Pins do not apply to [`ConflictKind::Binary`] or `FileWhole` overlaps —
//! those are hard conflicts where blending mods byte-for-byte is not
//! sound; load-order is the only knob.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Identifies one field-level conflict point. Matches the key used by
/// [`detect_conflicts`](super::conflicts::detect_conflicts) for `Field`
/// conflicts.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FieldKey {
    pub file_path: String,
    pub record_id: u32,
    pub field: String,
}

/// On-disk shape for [`ResolutionMap`] — a flat list, since JSON object
/// keys must be strings and our key is a struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Entry {
    key: FieldKey,
    mod_slug: String,
}

/// Pin: which mod's value should win for a given field key.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ResolutionMap {
    entries: BTreeMap<FieldKey, String>,
}

impl Serialize for ResolutionMap {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let list: Vec<Entry> = self
            .entries
            .iter()
            .map(|(k, v)| Entry {
                key: k.clone(),
                mod_slug: v.clone(),
            })
            .collect();
        list.serialize(ser)
    }
}

impl<'de> Deserialize<'de> for ResolutionMap {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let list: Vec<Entry> = Vec::deserialize(de)?;
        let mut entries = BTreeMap::new();
        for Entry { key, mod_slug } in list {
            entries.insert(key, mod_slug);
        }
        Ok(Self { entries })
    }
}

impl ResolutionMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Pin a key to `mod_slug`. Replaces any existing pin on that key.
    pub fn pin(&mut self, key: FieldKey, mod_slug: impl Into<String>) {
        self.entries.insert(key, mod_slug.into());
    }

    /// Remove the pin for `key`, if any. Returns the previously-pinned slug.
    pub fn unpin(&mut self, key: &FieldKey) -> Option<String> {
        self.entries.remove(key)
    }

    /// The slug pinned to `key`, if any.
    pub fn winner(&self, key: &FieldKey) -> Option<&str> {
        self.entries.get(key).map(String::as_str)
    }

    /// Drop every pin whose mod_slug is not present in `enabled`. Used after
    /// enable/disable / load-order changes to keep the map tidy and to make
    /// the conflict UI honest about which pins are still in force.
    ///
    /// Returns the number of entries dropped.
    pub fn prune_to(&mut self, enabled: &[String]) -> usize {
        let before = self.entries.len();
        self.entries.retain(|_, slug| enabled.iter().any(|e| e == slug));
        before - self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&FieldKey, &str)> {
        self.entries.iter().map(|(k, v)| (k, v.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(file: &str, rec: u32, field: &str) -> FieldKey {
        FieldKey {
            file_path: file.to_owned(),
            record_id: rec,
            field: field.to_owned(),
        }
    }

    #[test]
    fn pin_and_winner() {
        let mut m = ResolutionMap::new();
        m.pin(k("MiscItem.db", 42, "name"), "spelling");
        assert_eq!(m.winner(&k("MiscItem.db", 42, "name")), Some("spelling"));
        assert_eq!(m.winner(&k("MiscItem.db", 42, "price")), None);
    }

    #[test]
    fn unpin_returns_previous() {
        let mut m = ResolutionMap::new();
        let key = k("a", 0, "x");
        m.pin(key.clone(), "a-mod");
        assert_eq!(m.unpin(&key).as_deref(), Some("a-mod"));
        assert!(m.unpin(&key).is_none());
    }

    #[test]
    fn prune_drops_pins_to_disabled_mods() {
        let mut m = ResolutionMap::new();
        m.pin(k("a", 0, "x"), "alive");
        m.pin(k("a", 1, "x"), "dead");
        assert_eq!(m.prune_to(&["alive".into()]), 1);
        assert_eq!(m.len(), 1);
        assert_eq!(m.winner(&k("a", 0, "x")), Some("alive"));
    }

    #[test]
    fn serde_round_trip_is_object_keyed_by_field_key() {
        let mut m = ResolutionMap::new();
        m.pin(k("MiscItem.db", 42, "name"), "spelling");
        let json = serde_json::to_string(&m).unwrap();
        let back: ResolutionMap = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }
}
