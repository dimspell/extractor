//! Compute & cache "differs from vanilla" address sets.
//!
//! When a hex editor is opened against a workspace that has a vanilla
//! snapshot for the file (or against a game-dir file with no snapshot yet,
//! treating the on-disk content as the implicit baseline), we precompute the
//! set of addresses that already differ. The matrix overlay tints those
//! cells distinctly from "dirty" (= dirtied this session) so authors can
//! see what their mod has cumulatively changed.

use std::collections::BTreeSet;

/// Set of addresses where `current[i] != vanilla[i]`. Length differences
/// past `min(len)` are reported as differing addresses too.
pub fn compute_diff(vanilla: &[u8], current: &[u8]) -> BTreeSet<u64> {
    let mut out = BTreeSet::new();
    let common = vanilla.len().min(current.len());
    for i in 0..common {
        if vanilla[i] != current[i] {
            out.insert(i as u64);
        }
    }
    let extra = vanilla.len().max(current.len());
    for i in common..extra {
        out.insert(i as u64);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_buffers_have_empty_diff() {
        let a = vec![0, 1, 2, 3];
        assert!(compute_diff(&a, &a).is_empty());
    }

    #[test]
    fn single_byte_change_shows_one_addr() {
        let a = vec![0, 1, 2, 3];
        let b = vec![0, 9, 2, 3];
        let d = compute_diff(&a, &b);
        assert_eq!(d.len(), 1);
        assert!(d.contains(&1));
    }

    #[test]
    fn longer_current_marks_tail_as_diff() {
        let a = vec![0, 1];
        let b = vec![0, 1, 2, 3];
        let d = compute_diff(&a, &b);
        assert_eq!(d, BTreeSet::from([2, 3]));
    }

    #[test]
    fn shorter_current_marks_truncated_tail_as_diff() {
        let a = vec![0, 1, 2, 3];
        let b = vec![0, 1];
        let d = compute_diff(&a, &b);
        assert_eq!(d, BTreeSet::from([2, 3]));
    }
}
