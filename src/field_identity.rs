//! The canonical *in-process* identity of one logical game-data cell.
//!
//! The same cell used to be addressed four different ways:
//!
//! * [`crate::modding::resolution::FieldKey`] — flat `(file, record_id, field)`;
//! * [`crate::localization::TextEntry`] — `(file, usize record_id, &'static str field)`;
//! * patcher DSLs — event scripts pack `(section, element, subfield)` into a
//!   dotted string and party levels pack `(npc, level)` into one `u32`;
//! * byte offsets — generated `RecordLayout` metadata on one side and literal
//!   offsets in hand-written parsers/tests on the other.
//!
//! [`FieldAddress`] is the one typed representation that can express all of
//! these *without* packing nested coordinates into strings or a single
//! integer. The wire shapes above are intentionally left unchanged; this
//! module owns the conversions so every boundary funnels through one place:
//!
//! ```text
//!   FieldKey / TextEntry / patcher DSL  <---->  FieldAddress
//!        (flat, serde-stable)              (typed, nested-capable)
//! ```
//!
//! No on-disk format or user-visible JSON schema changes: converting a flat
//! key yields the identical `(file, record_id, field)` triple, and rendering a
//! nested address reproduces the legacy dotted / packed string verbatim.

use crate::modding::resolution::FieldKey;

pub use crate::references::layout::PARTY_LEVEL_RECORD as PARTY_LEVEL_LAYOUT;

/// Number of level rows per NPC in `PrtLevel.db`. This is the stride used
/// when a `(npc_index, level_index)` coordinate is packed into the flat
/// `record_id` the patcher trait exposes. Kept here so the patcher and the
/// typed address can never disagree on the packing.
pub const PARTY_LEVELS_PER_NPC: u32 = 20;

// ─── Errors ──────────────────────────────────────────────────────────────────

/// Why a legacy DSL string could not be interpreted as a typed field path.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FieldAddressError {
    #[error("empty field path")]
    Empty,
    #[error("empty field path segment")]
    EmptySegment,
    #[error("expected numeric index, got `{0}`")]
    InvalidIndex(String),
}

// ─── Field path (the nested part of the address) ─────────────────────────────

/// One segment of a nested field path.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PathSegment {
    /// A named component (section name, subfield name, …).
    Name(String),
    /// A zero-based positional component (element index, parameter index, …).
    Index(usize),
}

impl PathSegment {
    pub fn as_name(&self) -> Option<&str> {
        match self {
            PathSegment::Name(name) => Some(name),
            PathSegment::Index(_) => None,
        }
    }

    pub fn as_index(&self) -> Option<usize> {
        match self {
            PathSegment::Index(index) => Some(*index),
            PathSegment::Name(_) => None,
        }
    }

    /// Render as it appears in the legacy dotted DSL (`name` or `N`).
    pub fn text(&self) -> String {
        match self {
            PathSegment::Name(name) => name.clone(),
            PathSegment::Index(index) => index.to_string(),
        }
    }
}

/// Where a field lives *inside* one record.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FieldPath {
    /// A plain, flat field name — the common case (`"name"`, `"strength"`).
    Name(String),
    /// A nested path — e.g. event-script `variables.0.value` becomes
    /// `[Name("variables"), Index(0), Name("value")]`.
    Nested(Vec<PathSegment>),
}

impl FieldPath {
    pub fn name(name: impl Into<String>) -> Self {
        FieldPath::Name(name.into())
    }

    pub fn nested(segments: Vec<PathSegment>) -> Self {
        FieldPath::Nested(segments)
    }

    /// Interpret a legacy dotted field string as a typed path. Numeric
    /// segments become [`PathSegment::Index`]; everything else is a name.
    pub fn parse_dotted(s: &str) -> Result<Self, FieldAddressError> {
        if s.is_empty() {
            return Err(FieldAddressError::Empty);
        }
        let mut segments = Vec::new();
        for part in s.split('.') {
            if part.is_empty() {
                return Err(FieldAddressError::EmptySegment);
            }
            segments.push(match part.parse::<usize>() {
                Ok(index) => PathSegment::Index(index),
                Err(_) => PathSegment::Name(part.to_owned()),
            });
        }
        // A single named segment is the flat case; every other shape is nested.
        if segments.len() == 1 {
            return Ok(FieldPath::Name(segments.remove(0).text()));
        }
        Ok(FieldPath::Nested(segments))
    }

    /// Render back to the exact dotted string used by the patcher DSL and the
    /// `FieldDelta.field` JSON field.
    pub fn to_dotted(&self) -> String {
        match self {
            FieldPath::Name(name) => name.clone(),
            FieldPath::Nested(segments) => segments
                .iter()
                .map(PathSegment::text)
                .collect::<Vec<_>>()
                .join("."),
        }
    }

    pub fn as_name(&self) -> Option<&str> {
        match self {
            FieldPath::Name(name) => Some(name),
            FieldPath::Nested(_) => None,
        }
    }

    /// For nested paths shaped `section.N[...]`, return the section name, the
    /// element index and the remaining segments after the index. This is the
    /// shape every section of the event-script DSL uses.
    pub fn split_section_element(
        &self,
    ) -> Result<(&str, usize, &[PathSegment]), FieldAddressError> {
        let FieldPath::Nested(segments) = self else {
            return Err(FieldAddressError::Empty);
        };
        if segments.len() < 2 {
            return Err(FieldAddressError::Empty);
        }
        let section = segments[0]
            .as_name()
            .ok_or(FieldAddressError::EmptySegment)?;
        let index = segments[1]
            .as_index()
            .ok_or_else(|| FieldAddressError::InvalidIndex(segments[1].text()))?;
        Ok((section, index, &segments[2..]))
    }
}

// ─── Record reference (the within-file part of the address) ─────────────────

/// Which record inside a file is addressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RecordRef {
    /// A plain `record_id` — the flat case used by almost every format.
    Index(u32),
    /// A two-axis coordinate flattened into one `record_id` as
    /// `row * stride + col`. `col` is zero-based; for party levels `row` is
    /// the NPC index and `col` is `level - 1`.
    Grid { row: u32, col: u32, stride: u32 },
}

impl RecordRef {
    /// The flat `record_id` carried by `FieldKey` / `FieldDelta`.
    pub fn to_record_id(self) -> u32 {
        match self {
            RecordRef::Index(id) => id,
            RecordRef::Grid { row, col, stride } => row * stride + col,
        }
    }
}

// ─── The canonical address ──────────────────────────────────────────────────

/// One typed address, shared by modding, localization, recording and patchers.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldAddress {
    pub file_path: String,
    pub record: RecordRef,
    pub field: FieldPath,
}

impl FieldAddress {
    /// The flat `(file, record_id, field)` case.
    pub fn flat(file_path: impl Into<String>, record_id: u32, field: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            record: RecordRef::Index(record_id),
            field: FieldPath::Name(field.into()),
        }
    }

    /// Event scripts have a single record per file; their address is nested
    /// as `section.element.subfield` in the field path.
    pub fn event_script(
        file_path: impl Into<String>,
        section: impl Into<String>,
        element: usize,
        subfield: Option<&str>,
    ) -> Self {
        let mut segments = vec![
            PathSegment::Name(section.into()),
            PathSegment::Index(element),
        ];
        if let Some(subfield) = subfield {
            segments.push(PathSegment::Name(subfield.to_owned()));
        }
        Self {
            file_path: file_path.into(),
            record: RecordRef::Index(0),
            field: FieldPath::Nested(segments),
        }
    }

    /// Party levels pack `(npc_index, level_index)` into `record_id`.
    pub fn party_level(
        file_path: impl Into<String>,
        npc_index: u32,
        level_index: u32,
        field: impl Into<String>,
    ) -> Self {
        Self {
            file_path: file_path.into(),
            record: RecordRef::Grid {
                row: npc_index,
                col: level_index,
                stride: PARTY_LEVELS_PER_NPC,
            },
            field: FieldPath::Name(field.into()),
        }
    }

    /// Decode a party-level flat `record_id` into its typed grid address.
    pub fn from_party_level_record_id(
        file_path: impl Into<String>,
        record_id: u32,
        field: impl Into<String>,
    ) -> Self {
        Self::party_level(
            file_path,
            record_id / PARTY_LEVELS_PER_NPC,
            record_id % PARTY_LEVELS_PER_NPC,
            field,
        )
    }

    /// Interpret a flat wire key, parsing a dotted `field` as a nested path.
    /// Unlike [`FieldAddress::from`] (which always yields a flat [`FieldPath::Name`]),
    /// this recovers the nested structure for the patcher DSL round-trip.
    pub fn from_field_key_parsed(key: &FieldKey) -> Result<Self, FieldAddressError> {
        Ok(Self {
            file_path: key.file_path.clone(),
            record: RecordRef::Index(key.record_id),
            field: FieldPath::parse_dotted(&key.field)?,
        })
    }

    /// The flattened, serde-stable wire key. Nested coordinates are rendered
    /// to the legacy dotted string / packed `record_id`.
    pub fn to_field_key(&self) -> FieldKey {
        FieldKey::new(
            self.file_path.clone(),
            self.record.to_record_id(),
            self.field.to_dotted(),
        )
    }
}

impl From<FieldKey> for FieldAddress {
    fn from(key: FieldKey) -> Self {
        Self {
            file_path: key.file_path,
            record: RecordRef::Index(key.record_id),
            field: FieldPath::Name(key.field),
        }
    }
}

impl From<FieldAddress> for FieldKey {
    fn from(address: FieldAddress) -> Self {
        address.to_field_key()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::localization::{TextEncoding, TextEntry};
    use crate::modding::change::FieldChange;
    use crate::modding::value::Value;

    const REL: &str = "CharacterInGame/MiscItem.db";

    // ── (a) localization entry ≡ modding change ─────────────────────────────

    fn text_entry(field: &'static str) -> TextEntry {
        TextEntry {
            file_path: REL.to_owned(),
            record_id: 7,
            field_name: field,
            original: "Helt".to_owned(),
            translation: "Helmet".to_owned(),
            encoding: TextEncoding::Windows1250,
            max_bytes: 30,
        }
    }

    #[test]
    fn test_localization_entry_and_modding_change_resolve_to_same_address() {
        let entry = text_entry("name");
        let change = FieldChange::new(
            FieldKey::new(REL, 7, "name"),
            Value::String("Helt".into()),
            Value::String("Helmet".into()),
        );
        assert_eq!(entry.field_address(), change.key.field_address());
        // Both flatten to the identical wire key — the equality conflict
        // detection, pins and apply all rely on.
        assert_eq!(entry.field_address().to_field_key(), change.key);
    }

    // ── (b) nested addresses round-trip through the DSL ─────────────────────

    #[test]
    fn test_event_script_address_round_trips_through_dotted_dsl() {
        let address =
            FieldAddress::event_script("Script/Eventcat1.scr", "variables", 0, Some("value"));
        let key = address.to_field_key();
        assert_eq!(key.record_id, 0);
        assert_eq!(key.field, "variables.0.value");

        let recovered = FieldAddress::from_field_key_parsed(&key).unwrap();
        assert_eq!(recovered, address);

        // The deeper `actions.N.parameters.M` shape keeps its extra index.
        let param = FieldAddress::event_script("Script/Eventcat1.scr", "actions", 2, None);
        let FieldPath::Nested(mut segments) = param.field.clone() else {
            panic!("expected nested path")
        };
        segments.push(PathSegment::Name("parameters".into()));
        segments.push(PathSegment::Index(1));
        let param = FieldAddress {
            field: FieldPath::Nested(segments),
            ..param
        };
        let key = param.to_field_key();
        assert_eq!(key.field, "actions.2.parameters.1");
        assert_eq!(FieldAddress::from_field_key_parsed(&key).unwrap(), param);
    }

    #[test]
    fn test_party_level_address_round_trips_through_packed_record_id() {
        // NPC 3, level 7 → 3*20 + (7-1) = 66, exactly the patcher DSL.
        let address = FieldAddress::party_level("NpcInGame/PrtLevel.db", 3, 6, "strength");
        let key = address.to_field_key();
        assert_eq!(key.record_id, 66);
        assert_eq!(key.field, "strength");

        let recovered =
            FieldAddress::from_party_level_record_id(&key.file_path, key.record_id, &key.field);
        assert_eq!(recovered, address);

        // The patcher's own decoder agrees with the typed address.
        let RecordRef::Grid { row, col, stride } = recovered.record else {
            panic!("expected grid reference")
        };
        assert_eq!((row, col, stride), (3, 6, PARTY_LEVELS_PER_NPC));
    }

    #[test]
    fn test_nested_address_rejects_non_numeric_element() {
        let key = FieldKey::new("Script/Eventcat1.scr", 0, "variables.foo.value");
        let address = FieldAddress::from_field_key_parsed(&key).unwrap();
        let err = address.field.split_section_element().unwrap_err();
        assert_eq!(err, FieldAddressError::InvalidIndex("foo".into()));
        assert!(err.to_string().contains("numeric index"));
    }

    // ── offset helpers ──────────────────────────────────────────────────────

    #[test]
    fn test_party_level_layout_exposes_field_offsets() {
        let layout = PARTY_LEVEL_LAYOUT;
        assert_eq!(layout.record_size, 36);
        assert_eq!(layout.field_offset("accuracy"), Some(20));
        assert_eq!(layout.field_offset("tactical_action_chance"), Some(32));
        assert_eq!(layout.field("strength").map(|f| f.size), Some(4));
        assert_eq!(layout.field_offset("level"), None); // derived, not on disk
    }
}
