//! Pristine-game-file snapshot store.
//!
//! Backs the apply engine: before any mod first touches a file, a copy of
//! that file is captured under `<workspace>/vanilla/<relative_path>`.
//! Disabling or reordering mods means re-applying from vanilla, so the
//! original game directory is always recoverable.
//!
//! Files that did *not* exist in the pristine game (e.g. introduced by a
//! `FileAdd`) are *not* snapshotted; their absence is represented implicitly
//! by [`VanillaStore::read`] returning `Ok(None)`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::error::{ModdingError, Result};
use super::fs::{Fs, FsHandle, OsFs};

/// On-disk vanilla mirror rooted at a single workspace directory.
///
/// Storage goes through an [`Fs`] port so the facade can be exercised with an
/// in-memory stand-in; the default is the real filesystem.
#[derive(Clone)]
pub struct VanillaStore {
    root: PathBuf,
    fs: FsHandle,
}

impl std::fmt::Debug for VanillaStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VanillaStore")
            .field("root", &self.root)
            .finish_non_exhaustive()
    }
}

impl VanillaStore {
    pub fn new(root: PathBuf) -> Self {
        Self::with_fs(root, Arc::new(OsFs))
    }

    /// Build a store backed by a custom filesystem port.
    pub fn with_fs(root: PathBuf, fs: FsHandle) -> Self {
        Self { root, fs }
    }

    /// The filesystem port this store uses.
    pub fn fs(&self) -> &dyn Fs {
        self.fs.as_ref()
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Path inside the vanilla mirror for a relative game path.
    fn vanilla_path(&self, relative: &str) -> PathBuf {
        self.root.join(normalise(relative))
    }

    /// True iff a snapshot exists for `relative`.
    pub fn has(&self, relative: &str) -> bool {
        self.fs.is_file(&self.vanilla_path(relative))
    }

    /// Read a previously captured snapshot.
    ///
    /// Returns `Ok(None)` when no snapshot exists — meaning the file did not
    /// exist in pristine game state (or was never touched by a mod).
    pub fn read(&self, relative: &str) -> Result<Option<Vec<u8>>> {
        let path = self.vanilla_path(relative);
        if !self.fs.exists(&path) {
            return Ok(None);
        }
        Ok(Some(self.fs.read(&path)?))
    }

    /// Snapshot `relative` from `game_dir` if not already captured.
    ///
    /// - If a snapshot already exists, returns its bytes (`Ok(Some(_))`).
    /// - If no snapshot exists and the file is present in `game_dir`, copies
    ///   it into the mirror and returns the bytes.
    /// - If no snapshot exists and the file is absent in `game_dir`, returns
    ///   `Ok(None)` and writes nothing — the pristine state is "absent".
    pub fn ensure_snapshot(&self, game_dir: &Path, relative: &str) -> Result<Option<Vec<u8>>> {
        if let Some(bytes) = self.read(relative)? {
            return Ok(Some(bytes));
        }

        let src = game_dir.join(normalise(relative));
        if !self.fs.exists(&src) {
            return Ok(None);
        }

        let bytes = self.fs.read(&src)?;
        let dst = self.vanilla_path(relative);
        if let Some(parent) = dst.parent() {
            self.fs.create_dir_all(parent)?;
        }
        self.fs.write(&dst, &bytes)?;
        Ok(Some(bytes))
    }
}

/// Reject path-traversal attempts so a malicious mod can't escape `game_dir`.
fn normalise(relative: &str) -> PathBuf {
    Path::new(relative)
        .components()
        .filter(|c| matches!(c, std::path::Component::Normal(_)))
        .collect()
}

/// Validate that a relative path is safe to use against `game_dir`.
///
/// Used by the apply engine before *writing* into the game directory. The
/// vanilla store itself defends with [`normalise`], but apply also checks
/// upfront so callers get a clear error rather than a silently-rerouted write.
pub fn validate_relative(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(ModdingError::Malformed("empty file path".into()));
    }
    let p = Path::new(path);
    if p.is_absolute() {
        return Err(ModdingError::Malformed(format!(
            "absolute path not allowed: {path}"
        )));
    }
    for c in p.components() {
        if matches!(c, std::path::Component::ParentDir) {
            return Err(ModdingError::Malformed(format!(
                "path traversal not allowed: {path}"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn snapshot_then_read() {
        let game = tempdir().unwrap();
        let vault = tempdir().unwrap();
        let store = VanillaStore::new(vault.path().to_path_buf());

        let rel = "CharacterInGame/MiscItem.db";
        let abs = game.path().join(rel);
        fs::create_dir_all(abs.parent().unwrap()).unwrap();
        fs::write(&abs, b"original").unwrap();

        let snap = store.ensure_snapshot(game.path(), rel).unwrap();
        assert_eq!(snap.as_deref(), Some(&b"original"[..]));
        assert!(store.has(rel));

        // Mutate the source; snapshot must still return original bytes.
        fs::write(&abs, b"mutated").unwrap();
        let again = store.ensure_snapshot(game.path(), rel).unwrap();
        assert_eq!(again.as_deref(), Some(&b"original"[..]));
    }

    #[test]
    fn missing_source_returns_none() {
        let game = tempdir().unwrap();
        let vault = tempdir().unwrap();
        let store = VanillaStore::new(vault.path().to_path_buf());

        let rel = "Map/never_existed.map";
        let snap = store.ensure_snapshot(game.path(), rel).unwrap();
        assert!(snap.is_none());
        assert!(!store.has(rel));
    }

    #[test]
    fn rejects_path_traversal() {
        assert!(validate_relative("").is_err());
        assert!(validate_relative("/etc/passwd").is_err());
        assert!(validate_relative("../escape.txt").is_err());
        assert!(validate_relative("foo/../../bar").is_err());
        assert!(validate_relative("CharacterInGame/MiscItem.db").is_ok());
    }

    #[test]
    fn normalise_strips_leading_dots() {
        assert_eq!(normalise("./foo/bar"), PathBuf::from("foo/bar"));
        // ParentDir components are stripped (defence-in-depth).
        assert_eq!(normalise("foo/../bar"), PathBuf::from("foo/bar"));
    }
}
