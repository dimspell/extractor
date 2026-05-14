//! Thin wrappers around [`qbsdiff`] used by the apply engine and (later)
//! the recording pipeline.
//!
//! Patches produced by [`make_delta`] are always against the **vanilla**
//! bytes, never against intermediate state. The apply engine relies on this
//! so that disabling a mod and re-applying re-derives a clean result.

use std::io::Cursor;

use qbsdiff::{Bsdiff, Bspatch};

use super::error::{ModdingError, Result};

/// Build a binary delta from `source` (vanilla) to `target` (modded).
pub fn make_delta(source: &[u8], target: &[u8]) -> Result<Vec<u8>> {
    let mut patch = Vec::new();
    Bsdiff::new(source, target)
        .compare(Cursor::new(&mut patch))
        .map_err(|e| ModdingError::Malformed(format!("bsdiff failed: {e}")))?;
    Ok(patch)
}

/// Apply a delta produced by [`make_delta`] against `source` (vanilla).
pub fn apply_delta(source: &[u8], patch: &[u8]) -> Result<Vec<u8>> {
    let bp =
        Bspatch::new(patch).map_err(|e| ModdingError::Malformed(format!("bspatch parse: {e}")))?;
    let mut out = Vec::with_capacity(bp.hint_target_size() as usize);
    bp.apply(source, Cursor::new(&mut out))
        .map_err(|e| ModdingError::Malformed(format!("bspatch apply: {e}")))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_small() {
        let src = b"Hello, world!";
        let dst = b"Hello, brave new world!";
        let patch = make_delta(src, dst).unwrap();
        let back = apply_delta(src, &patch).unwrap();
        assert_eq!(back, dst);
    }

    #[test]
    fn round_trip_random_largeish() {
        // 8 KiB of pseudo-random data, then a small mutation.
        let src: Vec<u8> = (0u32..8192).map(|i| (i.wrapping_mul(31)) as u8).collect();
        let mut dst = src.clone();
        for byte in dst.iter_mut().skip(100).take(50) {
            *byte = byte.wrapping_add(7);
        }
        let patch = make_delta(&src, &dst).unwrap();
        assert!(
            patch.len() < src.len() / 4,
            "delta should be small for a tiny mutation"
        );
        let back = apply_delta(&src, &patch).unwrap();
        assert_eq!(back, dst);
    }

    #[test]
    fn malformed_patch_errors() {
        let err = apply_delta(b"src", b"not-a-patch").unwrap_err();
        assert!(matches!(err, ModdingError::Malformed(_)));
    }
}
