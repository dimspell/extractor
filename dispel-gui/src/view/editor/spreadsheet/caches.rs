//! Background-computable caches: display strings, row hashes, validation
//! flags. Computing these is `O(catalog × columns)` and dominates the
//! scroll-to-load latency on large catalogs, so the build is split off
//! from the install:
//!
//! * [`compute_caches`] is pure and allocation-only — safe to call from a
//!   `tokio::task::spawn_blocking` worker.
//! * [`SpreadsheetState::install_caches`](super::SpreadsheetState::install_caches)
//!   is the cheap UI-thread mutation that swaps the result in.

use crate::components::editable::EditableRecord;
use std::hash::{Hash, Hasher};

/// Pre-computed display data for an entire catalog.
///
/// All vectors are `catalog.len()` long and indexed by `orig_idx`.
#[derive(Debug, Clone, Default)]
pub struct ComputedCaches {
    pub row_hashes: Vec<u64>,
    pub display_cache: Vec<Vec<String>>,
    pub validation_cache: Vec<Vec<bool>>,
}

/// Walk every record, materialise display strings, hash them, and run
/// per-field validation. Pure — does not touch UI state or globals — so
/// safe to run from a background thread.
pub fn compute_caches<R: EditableRecord>(catalog: &[R]) -> ComputedCaches {
    let descriptors = R::field_descriptors();
    let num_cols = descriptors.len();
    let mut row_hashes = Vec::with_capacity(catalog.len());
    let mut display_cache = Vec::with_capacity(catalog.len());
    let mut validation_cache = Vec::with_capacity(catalog.len());

    for record in catalog.iter() {
        let values: Vec<String> = (0..num_cols)
            .map(|j| record.get_field(descriptors[j].name))
            .collect();
        let mut h = std::collections::hash_map::DefaultHasher::new();
        values.hash(&mut h);
        row_hashes.push(h.finish());

        let validation: Vec<bool> = (0..num_cols)
            .map(|j| {
                record
                    .validate_field(descriptors[j].name, &values[j])
                    .is_some()
            })
            .collect();

        display_cache.push(values);
        validation_cache.push(validation);
    }

    ComputedCaches {
        row_hashes,
        display_cache,
        validation_cache,
    }
}
