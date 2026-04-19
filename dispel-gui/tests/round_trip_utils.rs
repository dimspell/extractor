//! Round-trip integrity test utilities
//!
//! Provides helper functions for testing that extract→patch cycles produce
//! byte-identical output.

use std::path::Path;

/// Compare original and round-tripped bytes, providing detailed error on mismatch
pub fn round_trip_eq(original: &[u8], roundtripped: &[u8], type_name: &str) -> Result<(), String> {
    if original == roundtripped {
        Ok(())
    } else {
        let first_diff = original
            .iter()
            .zip(roundtripped.iter())
            .position(|(a, b)| a != b)
            .unwrap_or(0);

        let context_start = first_diff.saturating_sub(16);
        let context_end = std::cmp::min(original.len(), first_diff + 32);

        let original_ctx = &original[context_start..context_end];
        let roundtripped_ctx = &roundtripped[context_start..context_end];

        Err(format!(
            "Round-trip mismatch for {}\n\
             First difference at byte {}\n\
             Original bytes (offset {}): {:02x?}\n\
             Round-tripped bytes (offset {}): {:02x?}",
            type_name, first_diff, context_start, original_ctx, context_start, roundtripped_ctx
        ))
    }
}

/// Perform a round-trip test on a fixture file
pub fn round_trip_from_fixture<T>(
    load_fn: impl FnOnce(&Path) -> std::io::Result<Vec<T>>,
    save_fn: impl FnOnce(&[T], &Path) -> std::io::Result<()>,
    fixture_path: &Path,
    type_name: &str,
) -> Result<(), String>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    let temp_file = tempfile::NamedTempFile::new().map_err(|e| e.to_string())?;
    let temp_path = temp_file.path();

    let original_bytes = std::fs::read(fixture_path).map_err(|e| {
        format!(
            "Failed to read original fixture {}: {}",
            fixture_path.display(),
            e
        )
    })?;

    let records = load_fn(fixture_path).map_err(|e| {
        format!(
            "Failed to load {} from {}: {}",
            type_name,
            fixture_path.display(),
            e
        )
    })?;

    save_fn(&records, temp_path)
        .map_err(|e| format!("Failed to save {} to temp file: {}", type_name, e))?;

    let roundtripped_bytes = std::fs::read(temp_path).map_err(|e| e.to_string())?;

    round_trip_eq(&original_bytes, &roundtripped_bytes, type_name)
}
