//! Filesystem port for the modding engine.
//!
//! The deep modding facade ([`super::facade::Modding`]) owns a single
//! [`Fs`] value for *all* of its storage: the workspace metadata files,
//! installed mod packages, the vanilla mirror and the game directory.
//! Production code supplies [`OsFs`]; tests supply [`MemFs`], so the whole
//! pipeline can be exercised without touching a real disk.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use super::error::Result;

/// Minimal, portable view of the metadata modding cares about.
///
/// A struct (rather than `std::fs::Metadata`) is used so an in-memory
/// stand-in can report file sizes and kinds too.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FsMetadata {
    pub len: u64,
    pub is_file: bool,
    pub is_dir: bool,
}

/// Filesystem abstraction used by every I/O path in the modding engine.
///
/// Deliberately small: only the operations the pipeline actually needs, so a
/// test double is trivial to implement. [`OsFs`] is the sole production
/// adapter; [`MemFs`] is a complete in-memory implementation with the same
/// semantics.
pub trait Fs: Send + Sync {
    fn read(&self, path: &Path) -> Result<Vec<u8>>;
    fn write(&self, path: &Path, bytes: &[u8]) -> Result<()>;
    fn create_dir_all(&self, path: &Path) -> Result<()>;
    fn remove_file(&self, path: &Path) -> Result<()>;
    fn remove_dir_all(&self, path: &Path) -> Result<()>;
    /// Copy a file, returning the number of bytes copied.
    fn copy(&self, from: &Path, to: &Path) -> Result<u64>;
    /// Move a file or a whole directory tree.
    fn rename(&self, from: &Path, to: &Path) -> Result<()>;
    fn metadata(&self, path: &Path) -> Result<FsMetadata>;
    fn exists(&self, path: &Path) -> bool;
    fn is_file(&self, path: &Path) -> bool;
    fn is_dir(&self, path: &Path) -> bool;
    /// Immediate children of `path`, both files and directories.
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
}

/// The real filesystem — the only place in `modding` that calls `std::fs`.
#[derive(Debug, Default, Clone, Copy)]
pub struct OsFs;

impl Fs for OsFs {
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        Ok(std::fs::read(path)?)
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        Ok(std::fs::write(path, bytes)?)
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        Ok(std::fs::create_dir_all(path)?)
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        Ok(std::fs::remove_file(path)?)
    }

    fn remove_dir_all(&self, path: &Path) -> Result<()> {
        Ok(std::fs::remove_dir_all(path)?)
    }

    fn copy(&self, from: &Path, to: &Path) -> Result<u64> {
        Ok(std::fs::copy(from, to)?)
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        Ok(std::fs::rename(from, to)?)
    }

    fn metadata(&self, path: &Path) -> Result<FsMetadata> {
        let meta = std::fs::metadata(path)?;
        Ok(FsMetadata {
            len: meta.len(),
            is_file: meta.is_file(),
            is_dir: meta.is_dir(),
        })
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(path)? {
            out.push(entry?.path());
        }
        Ok(out)
    }
}

/// Shared handle used by every modding component so a facade and its test
/// doubles agree on storage.
pub type FsHandle = Arc<dyn Fs>;

fn not_found(path: &Path) -> super::error::ModdingError {
    std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("not found: {}", path.display()),
    )
    .into()
}

/// In-memory filesystem with the same observable behaviour as [`OsFs`].
///
/// Files hold bytes; directories are tracked explicitly (so an empty
/// directory created with `create_dir_all` still exists) and additionally as
/// implicit prefixes of stored paths. Paths are normalised through
/// [`Path::components`] before use, so `a/./b` and `a/b` address the same
/// entry.
#[derive(Debug, Default)]
pub struct MemFs {
    files: RwLock<HashMap<PathBuf, Vec<u8>>>,
    dirs: RwLock<BTreeSet<PathBuf>>,
}

impl MemFs {
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of files currently stored (directories are not counted).
    pub fn file_count(&self) -> usize {
        self.files.read().unwrap().len()
    }

    fn key(path: &Path) -> PathBuf {
        path.components().collect()
    }

    /// Record `path` and all its ancestors as directories.
    fn note_dirs(&self, path: &Path) {
        let key = Self::key(path);
        let mut dirs = self.dirs.write().unwrap();
        for ancestor in key.ancestors() {
            if !ancestor.as_os_str().is_empty() {
                dirs.insert(ancestor.to_path_buf());
            }
        }
    }

    /// The immediate child of `prefix` that contains `path`, if any.
    fn child(prefix: &Path, path: &Path) -> Option<PathBuf> {
        if path == prefix || !path.starts_with(prefix) {
            return None;
        }
        let component = path.components().nth(prefix.components().count())?;
        Some(prefix.join(component.as_os_str()))
    }
}

impl Fs for MemFs {
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        let key = Self::key(path);
        self.files
            .read()
            .unwrap()
            .get(&key)
            .cloned()
            .ok_or_else(|| not_found(path))
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        if let Some(parent) = path.parent() {
            self.note_dirs(parent);
        }
        self.files
            .write()
            .unwrap()
            .insert(Self::key(path), bytes.to_vec());
        Ok(())
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        self.note_dirs(path);
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        let key = Self::key(path);
        if self.files.write().unwrap().remove(&key).is_none() {
            return Err(not_found(path));
        }
        Ok(())
    }

    fn remove_dir_all(&self, path: &Path) -> Result<()> {
        if !self.is_dir(path) {
            return Err(not_found(path));
        }
        let key = Self::key(path);
        self.files
            .write()
            .unwrap()
            .retain(|p, _| !(p.starts_with(&key) && p != &key));
        self.dirs
            .write()
            .unwrap()
            .retain(|d| d != &key && !d.starts_with(&key));
        Ok(())
    }

    fn copy(&self, from: &Path, to: &Path) -> Result<u64> {
        let from_key = Self::key(from);
        let bytes = self
            .files
            .read()
            .unwrap()
            .get(&from_key)
            .cloned()
            .ok_or_else(|| not_found(from))?;
        if let Some(parent) = to.parent() {
            self.note_dirs(parent);
        }
        let len = bytes.len() as u64;
        self.files.write().unwrap().insert(Self::key(to), bytes);
        Ok(len)
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        let from_key = Self::key(from);
        let to_key = Self::key(to);
        if let Some(parent) = to.parent() {
            self.note_dirs(parent);
        }

        let is_file = self.files.read().unwrap().contains_key(&from_key);
        if is_file {
            let bytes = self
                .files
                .write()
                .unwrap()
                .remove(&from_key)
                .ok_or_else(|| not_found(from))?;
            self.files.write().unwrap().insert(to_key, bytes);
            return Ok(());
        }
        if !self.is_dir(from) {
            return Err(not_found(from));
        }

        // Move every file under the source prefix, preserving relative paths.
        let moving: Vec<(PathBuf, Vec<u8>)> = {
            let files = self.files.read().unwrap();
            files
                .iter()
                .filter(|(p, _)| p.starts_with(&from_key))
                .map(|(p, b)| (p.clone(), b.clone()))
                .collect()
        };
        {
            let mut files = self.files.write().unwrap();
            for (old, _) in &moving {
                files.remove(old);
            }
            for (old, bytes) in moving {
                let rel = old.strip_prefix(&from_key).expect("prefix checked");
                files.insert(to_key.join(rel), bytes);
            }
        }
        // Move the directory entries themselves.
        let moving_dirs: Vec<PathBuf> = {
            let dirs = self.dirs.read().unwrap();
            dirs.iter()
                .filter(|d| d.starts_with(&from_key))
                .cloned()
                .collect()
        };
        {
            let mut dirs = self.dirs.write().unwrap();
            for old in moving_dirs {
                dirs.remove(&old);
                let rel = old.strip_prefix(&from_key).expect("prefix checked");
                let new = if rel.as_os_str().is_empty() {
                    to_key.clone()
                } else {
                    to_key.join(rel)
                };
                dirs.insert(new);
            }
        }
        Ok(())
    }

    fn metadata(&self, path: &Path) -> Result<FsMetadata> {
        let key = Self::key(path);
        let file_len = self.files.read().unwrap().get(&key).map(|b| b.len() as u64);
        if let Some(len) = file_len {
            return Ok(FsMetadata {
                len,
                is_file: true,
                is_dir: false,
            });
        }
        if self.is_dir(path) {
            return Ok(FsMetadata {
                len: 0,
                is_file: false,
                is_dir: true,
            });
        }
        Err(not_found(path))
    }

    fn exists(&self, path: &Path) -> bool {
        self.is_file(path) || self.is_dir(path)
    }

    fn is_file(&self, path: &Path) -> bool {
        self.files.read().unwrap().contains_key(&Self::key(path))
    }

    fn is_dir(&self, path: &Path) -> bool {
        let key = Self::key(path);
        if self.dirs.read().unwrap().contains(&key) {
            return true;
        }
        self.files
            .read()
            .unwrap()
            .keys()
            .any(|p| p.starts_with(&key) && p != &key)
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        if !self.is_dir(path) {
            return Err(not_found(path));
        }
        let prefix = Self::key(path);
        let mut children = BTreeSet::new();
        {
            let files = self.files.read().unwrap();
            for file in files.keys() {
                if let Some(child) = Self::child(&prefix, file) {
                    children.insert(child);
                }
            }
        }
        {
            let dirs = self.dirs.read().unwrap();
            for dir in dirs.iter() {
                if let Some(child) = Self::child(&prefix, dir) {
                    children.insert(child);
                }
            }
        }
        Ok(children.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mem_fs_round_trips_bytes() {
        let fs = MemFs::new();
        let path = Path::new("/game/MiscItem.db");
        fs.create_dir_all(Path::new("/game")).unwrap();
        fs.write(path, b"abc").unwrap();
        assert_eq!(fs.read(path).unwrap(), b"abc");
        assert!(fs.is_file(path));
        assert!(fs.exists(path));
        assert!(fs.is_dir(Path::new("/game")));
        fs.remove_file(path).unwrap();
        assert!(!fs.exists(path));
        assert_eq!(fs.file_count(), 0);
    }

    #[test]
    fn mem_fs_copy_reports_byte_count_and_is_independent() {
        let fs = MemFs::new();
        let src = Path::new("/game/a.bin");
        let dst = Path::new("/game/copy/b.bin");
        fs.write(src, b"hello").unwrap();
        assert_eq!(fs.copy(src, dst).unwrap(), 5);
        assert_eq!(fs.read(dst).unwrap(), b"hello");
        fs.write(src, b"changed").unwrap();
        assert_eq!(fs.read(dst).unwrap(), b"hello");
        assert!(fs.copy(Path::new("/game/missing"), dst).is_err());
    }

    #[test]
    fn mem_fs_rename_moves_files_and_directories() {
        let fs = MemFs::new();
        let src = Path::new("/mods/a/manifest.json");
        fs.write(src, b"{}").unwrap();
        fs.rename(src, Path::new("/mods/b/manifest.json")).unwrap();
        assert!(!fs.exists(src));
        assert!(fs.is_file(Path::new("/mods/b/manifest.json")));
        assert!(fs.rename(src, src).is_err());

        fs.write(Path::new("/mods/tree/a/x.bin"), b"x").unwrap();
        fs.write(Path::new("/mods/tree/b/y.bin"), b"y").unwrap();
        fs.rename(Path::new("/mods/tree"), Path::new("/mods/moved"))
            .unwrap();
        assert!(!fs.is_dir(Path::new("/mods/tree")));
        assert_eq!(fs.read(Path::new("/mods/moved/a/x.bin")).unwrap(), b"x");
        assert_eq!(fs.read(Path::new("/mods/moved/b/y.bin")).unwrap(), b"y");
    }

    #[test]
    fn mem_fs_remove_dir_all_is_recursive_and_errors_on_missing() {
        let fs = MemFs::new();
        fs.write(Path::new("/mods/a/manifest.json"), b"{}").unwrap();
        fs.write(Path::new("/mods/a/patches/p.bin"), b"p").unwrap();
        fs.write(Path::new("/mods/b/manifest.json"), b"{}").unwrap();
        fs.remove_dir_all(Path::new("/mods/a")).unwrap();
        assert!(!fs.exists(Path::new("/mods/a")));
        assert_eq!(fs.file_count(), 1);
        assert!(fs.is_file(Path::new("/mods/b/manifest.json")));
        assert!(fs.remove_dir_all(Path::new("/mods/ghost")).is_err());
    }

    #[test]
    fn mem_fs_metadata_reports_files_and_dirs() {
        let fs = MemFs::new();
        fs.write(Path::new("/game/x.db"), b"1234").unwrap();
        assert_eq!(
            fs.metadata(Path::new("/game/x.db")).unwrap(),
            FsMetadata {
                len: 4,
                is_file: true,
                is_dir: false
            }
        );
        let dir = fs.metadata(Path::new("/game")).unwrap();
        assert!(dir.is_dir && !dir.is_file);
        assert!(fs.metadata(Path::new("/game/absent")).is_err());
    }

    #[test]
    fn mem_fs_read_dir_lists_immediate_children() {
        let fs = MemFs::new();
        fs.write(Path::new("/mods/a/manifest.json"), b"{}").unwrap();
        fs.write(Path::new("/mods/a/patches/p.bin"), b"p").unwrap();
        assert_eq!(
            fs.read_dir(Path::new("/mods/a")).unwrap(),
            vec![
                PathBuf::from("/mods/a/manifest.json"),
                PathBuf::from("/mods/a/patches"),
            ]
        );
        assert!(fs.read_dir(Path::new("/mods/ghost")).is_err());
    }
}
