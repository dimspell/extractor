//! Field-level patching for catalog (`.db` / `.ini`) files.
//!
//! Each game catalog format gets a [`RecordPatcher`] implementation that
//! takes the file's full byte buffer plus a single field change and produces
//! the new byte buffer. The apply engine ([`super::apply`]) drives these.
//!
//! The trait works on bytes (not parsed records) so the engine can chain
//! multiple deltas through a single read/write cycle without forcing
//! callers to know the concrete record type.

use super::error::{ModdingError, Result};
use super::value::Value;

/// Knows how to apply [`ChangeOp::FieldDelta`](super::change::ChangeOp::FieldDelta)
/// to one specific catalog file format.
pub trait RecordPatcher: Send + Sync {
    /// Human-readable name (used in error messages and diagnostics).
    fn name(&self) -> &'static str;

    /// Replace one field on one record and return the new file bytes.
    fn apply_field(
        &self,
        bytes: &[u8],
        record_id: u32,
        field: &str,
        new: &Value,
    ) -> Result<Vec<u8>>;
}

/// Convenience constructor for "unknown field" errors.
pub fn unknown_field(record: &str, field: &str) -> ModdingError {
    ModdingError::Malformed(format!("{record}: unknown field `{field}`"))
}

/// Convenience constructor for "wrong value type" errors.
pub fn wrong_type(record: &str, field: &str, expected: &str, got: &Value) -> ModdingError {
    ModdingError::Malformed(format!(
        "{record}.{field}: expected {expected}, got {}",
        got.type_name()
    ))
}

/// Convenience constructor for "record id out of range" errors.
pub fn out_of_range(record: &str, id: u32, len: usize) -> ModdingError {
    ModdingError::Malformed(format!(
        "{record}: record_id {id} out of range (have {len})"
    ))
}
