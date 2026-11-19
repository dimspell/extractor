//! Filesystem port for the modding engine.
//!
//! The deep modding facade ([`super::facade::Modding`]) owns a single
//! [`Fs`] value. Production code supplies [`OsFs`]; tests can supply an
//! in-memory stand-in, so apply / vanilla / recording paths can be verified
//! without touching the real game directory.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::error::Result;
#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::RwLock;

/// Minimal filesystem abstraction used by the apply engine and the vanilla
/// mirror. Deliberately small: only the operations the pipeline actually
/// needs, so a test double is trivial to implement.
pub trait Fs: Send + Sync {
    fn read(&self, path: &Path) -> Result<Vec<u8>>;
    fn write(&self, path: &Path, bytes: &[u8]) -> Result<()>;
    fn create_dir_all(&self, path: &Path) -> Result<()>;
    fn remove_file(&self, path: &Path) -> Result<()>;
    fn exists(&self, path: &Path) -> bool;
    fn is_file(&self, path: &Path) -> bool;
    fn is_dir(&self, path: &Path) -> bool;
    /// Immediate children of `path`, both files and directories.
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
}

/// The real filesystem.
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

/// Shared handle used by [`VanillaStore`](super::vanilla::VanillaStore) and
/// the apply engine so a facade and its test doubles agree on storage.
pub type FsHandle = Arc<dyn Fs>;

/// In-memory filesystem used by the boundary tests. Only files are
/// modelled; directories are implicit prefixes of stored paths.
#[cfg(test)]
#[derive(Debug, Default)]
pub struct MemFs {
    files: RwLock<HashMap<PathBuf, Vec<u8>>>,
}

#[cfg(test)]
impl MemFs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn file_count(&self) -> usize {
        self.files.read().unwrap().len()
    }

    fn key(path: &Path) -> PathBuf {
        path.components().collect()
    }
}

#[cfg(test)]
impl Fs for MemFs {
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        self.files
            .read()
            .unwrap()
            .get(&Self::key(path))
            .cloned()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("not found in MemFs: {}", path.display()),
                )
                .into()
            })
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        self.files
            .write()
            .unwrap()
            .insert(Self::key(path), bytes.to_vec());
        Ok(())
    }

    fn create_dir_all(&self, _path: &Path) -> Result<()> {
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        self.files.write().unwrap().remove(&Self::key(path));
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        self.files.read().unwrap().contains_key(&Self::key(path))
    }

    fn is_file(&self, path: &Path) -> bool {
        self.exists(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        let prefix = Self::key(path);
        self.files
            .read()
            .unwrap()
            .keys()
            .any(|p| p.starts_with(&prefix) && p != &prefix)
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let prefix = Self::key(path);
        let mut children = std::collections::BTreeSet::new();
        for file in self.files.read().unwrap().keys() {
            if !file.starts_with(&prefix) || file == &prefix {
                continue;
            }
            let mut child = prefix.clone();
            if let Some(component) = file.components().nth(prefix.components().count()) {
                child.push(component.as_os_str());
                children.insert(child);
            }
        }
        Ok(children.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::{Fs, MemFs};
    use std::path::Path;

    #[test]
    fn mem_fs_round_trips_bytes() {
        let fs = MemFs::new();
        let path = Path::new("/game/MiscItem.db");
        fs.write(path, b"abc").unwrap();
        assert_eq!(fs.read(path).unwrap(), b"abc");
        assert!(fs.is_file(path));
        assert!(fs.is_dir(Path::new("/game")));
        fs.remove_file(path).unwrap();
        assert!(!fs.exists(path));
        assert_eq!(fs.file_count(), 0);
    }
}
