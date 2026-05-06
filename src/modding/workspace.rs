//! Filesystem layout that holds installed mods, the load-order list, and
//! the vanilla snapshot mirror.
//!
//! ```text
//! <root>/
//!   mods/
//!     <slug>/                  one directory per installed mod
//!       manifest.json
//!       changes.json
//!       patches/<uuid>.bin
//!       files/<uuid>.bin
//!   vanilla/                   pristine bytes captured before first patch
//!   enabled.json               ordered list of enabled mod slugs
//! ```
//!
//! `slug` is the directory name; it doubles as the mod's stable id. It is
//! derived from the manifest name on creation/import (lowercased, spaces
//! replaced with `-`, suffix `-2`, `-3`... appended on collision).

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::apply::{apply_all, revert_to_vanilla, ApplyReport, ModEntry, RevertReport};
use super::error::{ModdingError, Result};
use super::manifest::ModManifest;
use super::package::{self, ModPackage};
use super::registry::PatcherRegistry;
use super::vanilla::VanillaStore;

const MODS_DIR: &str = "mods";
const VANILLA_DIR: &str = "vanilla";
const ENABLED_FILE: &str = "enabled.json";

/// Lightweight summary of one installed mod, suitable for the Library list.
#[derive(Debug, Clone, PartialEq)]
pub struct InstalledMod {
    pub slug: String,
    pub manifest: ModManifest,
    pub change_count: usize,
    pub enabled: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct EnabledFile {
    /// Slugs in load order (first = applied first).
    enabled: Vec<String>,
}

/// Open or create a mod workspace at `root`.
#[derive(Debug, Clone)]
pub struct Workspace {
    root: PathBuf,
    vanilla: VanillaStore,
}

impl Workspace {
    /// Open (and create-on-first-use) a workspace rooted at `root`.
    pub fn open(root: PathBuf) -> Result<Self> {
        fs::create_dir_all(&root)?;
        fs::create_dir_all(root.join(MODS_DIR))?;
        let vanilla_root = root.join(VANILLA_DIR);
        fs::create_dir_all(&vanilla_root)?;
        Ok(Self {
            vanilla: VanillaStore::new(vanilla_root),
            root,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn vanilla(&self) -> &VanillaStore {
        &self.vanilla
    }

    fn mods_root(&self) -> PathBuf {
        self.root.join(MODS_DIR)
    }

    fn enabled_path(&self) -> PathBuf {
        self.root.join(ENABLED_FILE)
    }

    fn mod_dir(&self, slug: &str) -> Result<PathBuf> {
        validate_slug(slug)?;
        Ok(self.mods_root().join(slug))
    }

    /// Read the persisted load order. Slugs no longer present on disk are
    /// dropped silently.
    pub fn enabled_order(&self) -> Result<Vec<String>> {
        let path = self.enabled_path();
        if !path.is_file() {
            return Ok(Vec::new());
        }
        let raw: EnabledFile = serde_json::from_slice(&fs::read(&path)?)?;
        let installed: HashSet<String> = self.list_slugs()?.into_iter().collect();
        Ok(raw
            .enabled
            .into_iter()
            .filter(|s| installed.contains(s))
            .collect())
    }

    /// Persist `order` as the new load order. Unknown slugs are rejected.
    pub fn set_enabled_order(&self, order: Vec<String>) -> Result<()> {
        let installed: HashSet<String> = self.list_slugs()?.into_iter().collect();
        for slug in &order {
            if !installed.contains(slug) {
                return Err(ModdingError::Malformed(format!(
                    "unknown mod slug: {slug}"
                )));
            }
        }
        let raw = EnabledFile { enabled: order };
        fs::write(self.enabled_path(), serde_json::to_vec_pretty(&raw)?)?;
        Ok(())
    }

    /// Toggle a mod's enabled state. Newly-enabled mods go to the end of
    /// the load order; disabling preserves position for re-enabling.
    pub fn set_enabled(&self, slug: &str, enabled: bool) -> Result<()> {
        validate_slug(slug)?;
        let mut order = self.enabled_order()?;
        let in_order = order.iter().position(|s| s == slug);
        match (enabled, in_order) {
            (true, None) => order.push(slug.to_owned()),
            (false, Some(idx)) => {
                order.remove(idx);
            }
            _ => return Ok(()),
        }
        self.set_enabled_order(order)
    }

    /// Move a mod one position earlier in load order.
    pub fn move_up(&self, slug: &str) -> Result<()> {
        let mut order = self.enabled_order()?;
        if let Some(idx) = order.iter().position(|s| s == slug) {
            if idx > 0 {
                order.swap(idx - 1, idx);
                self.set_enabled_order(order)?;
            }
        }
        Ok(())
    }

    /// Move a mod one position later in load order.
    pub fn move_down(&self, slug: &str) -> Result<()> {
        let mut order = self.enabled_order()?;
        if let Some(idx) = order.iter().position(|s| s == slug) {
            if idx + 1 < order.len() {
                order.swap(idx, idx + 1);
                self.set_enabled_order(order)?;
            }
        }
        Ok(())
    }

    /// List every installed mod, with load-order and enabled flag attached.
    /// Enabled mods come first in their persisted load order; disabled mods
    /// follow alphabetically.
    pub fn list_mods(&self) -> Result<Vec<InstalledMod>> {
        let slugs = self.list_slugs()?;
        let order = self.enabled_order()?;
        let order_set: HashSet<&str> = order.iter().map(String::as_str).collect();

        let mut result = Vec::with_capacity(slugs.len());

        for slug in &order {
            if let Some(installed) = self.summary(slug)? {
                result.push(InstalledMod {
                    enabled: true,
                    ..installed
                });
            }
        }
        let mut disabled: Vec<_> = slugs
            .iter()
            .filter(|s| !order_set.contains(s.as_str()))
            .collect();
        disabled.sort();
        for slug in disabled {
            if let Some(installed) = self.summary(slug)? {
                result.push(InstalledMod {
                    enabled: false,
                    ..installed
                });
            }
        }

        Ok(result)
    }

    fn summary(&self, slug: &str) -> Result<Option<InstalledMod>> {
        let dir = self.mod_dir(slug)?;
        if !dir.is_dir() {
            return Ok(None);
        }
        let pkg = package::read_dir(&dir)?;
        Ok(Some(InstalledMod {
            slug: slug.to_owned(),
            change_count: pkg.changes.len(),
            manifest: pkg.manifest,
            enabled: false, // caller fills this in
        }))
    }

    fn list_slugs(&self) -> Result<Vec<String>> {
        let mods = self.mods_root();
        if !mods.is_dir() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for entry in fs::read_dir(&mods)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            if let Some(name) = entry.file_name().to_str() {
                out.push(name.to_owned());
            }
        }
        Ok(out)
    }

    /// Read a mod package (manifest + full ChangeLog with blobs) by slug.
    pub fn read_mod(&self, slug: &str) -> Result<ModPackage> {
        package::read_dir(&self.mod_dir(slug)?)
    }

    /// Persist a mod package to its directory. The slug is taken as-is; use
    /// [`Self::create_mod`] if you need slug derivation.
    pub fn write_mod(&self, slug: &str, package: &ModPackage) -> Result<()> {
        package::write_dir(&self.mod_dir(slug)?, package)
    }

    /// Create a brand-new mod with the given manifest. Returns the assigned
    /// slug (derived from the manifest name with collision suffixing).
    pub fn create_mod(&self, manifest: ModManifest) -> Result<String> {
        let slug = self.allocate_slug(&manifest.name)?;
        let pkg = ModPackage::new(manifest, super::ChangeLog::new());
        self.write_mod(&slug, &pkg)?;
        Ok(slug)
    }

    /// Append one [`ChangeAction`](super::change::ChangeAction) to the named
    /// mod, in load-mutate-save fashion. Used by recording mode.
    pub fn append_action(
        &self,
        slug: &str,
        action: super::change::ChangeAction,
    ) -> Result<()> {
        let mut pkg = self.read_mod(slug)?;
        pkg.changes.push(action);
        self.write_mod(slug, &pkg)
    }

    /// Append many actions in one read-mutate-write cycle, then collapse
    /// repeated FieldDelta entries on the same `(file, record_id, field)`.
    /// Returns the number of actions removed by the flatten pass.
    pub fn append_actions_and_flatten(
        &self,
        slug: &str,
        actions: Vec<super::change::ChangeAction>,
    ) -> Result<usize> {
        let mut pkg = self.read_mod(slug)?;
        for a in actions {
            pkg.changes.push(a);
        }
        let removed = pkg.changes.flatten_field_deltas();
        self.write_mod(slug, &pkg)?;
        Ok(removed)
    }

    pub fn delete_mod(&self, slug: &str) -> Result<()> {
        let dir = self.mod_dir(slug)?;
        if dir.is_dir() {
            fs::remove_dir_all(&dir)?;
        }
        // Also drop from load order.
        let order: Vec<String> = self
            .enabled_order()?
            .into_iter()
            .filter(|s| s != slug)
            .collect();
        self.set_enabled_order(order)?;
        Ok(())
    }

    /// Import a `.zip` package. The zip is extracted into `mods/<slug>/`.
    pub fn import_zip(&self, zip_path: &Path) -> Result<String> {
        let file = fs::File::open(zip_path)?;
        let pkg = package::read_zip(file)?;
        let slug = self.allocate_slug(&pkg.manifest.name)?;
        self.write_mod(&slug, &pkg)?;
        Ok(slug)
    }

    /// Export an installed mod to `dst` (a `.zip` file path).
    pub fn export_zip(&self, slug: &str, dst: &Path) -> Result<()> {
        let pkg = self.read_mod(slug)?;
        let file = fs::File::create(dst)?;
        package::write_zip(file, &pkg)
    }

    /// Apply all enabled mods to `game_dir`.
    pub fn apply(&self, game_dir: &Path, registry: &PatcherRegistry) -> Result<ApplyReport> {
        let order = self.enabled_order()?;
        let packages: Vec<ModPackage> = order
            .iter()
            .map(|slug| self.read_mod(slug))
            .collect::<Result<Vec<_>>>()?;
        let mods: Vec<ModEntry<'_>> = order
            .iter()
            .zip(packages.iter())
            .map(|(slug, pkg)| ModEntry {
                mod_id: slug.as_str(),
                changes: &pkg.changes,
            })
            .collect();
        apply_all(&mods, game_dir, &self.vanilla, registry)
    }

    /// Restore every snapshotted file back to vanilla state.
    pub fn revert(&self, game_dir: &Path) -> Result<RevertReport> {
        revert_to_vanilla(game_dir, &self.vanilla)
    }

    fn allocate_slug(&self, name: &str) -> Result<String> {
        let base = slugify(name);
        if base.is_empty() {
            return Err(ModdingError::Malformed("mod name is empty".into()));
        }
        let installed: HashSet<String> = self.list_slugs()?.into_iter().collect();
        if !installed.contains(&base) {
            return Ok(base);
        }
        for n in 2..1000 {
            let candidate = format!("{base}-{n}");
            if !installed.contains(&candidate) {
                return Ok(candidate);
            }
        }
        Err(ModdingError::Malformed(
            "could not allocate a unique slug".into(),
        ))
    }
}

fn slugify(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut last_dash = true;
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_matches('-').to_owned()
}

fn validate_slug(slug: &str) -> Result<()> {
    if slug.is_empty()
        || slug
            .chars()
            .any(|c| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
    {
        return Err(ModdingError::Malformed(format!("invalid slug: {slug}")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modding::change::{ChangeAction, ChangeOp};
    use crate::modding::value::Value;
    use tempfile::tempdir;

    fn ws() -> (tempfile::TempDir, Workspace) {
        let dir = tempdir().unwrap();
        let ws = Workspace::open(dir.path().to_path_buf()).unwrap();
        (dir, ws)
    }

    #[test]
    fn open_creates_subdirs() {
        let (root, ws) = ws();
        assert!(root.path().join("mods").is_dir());
        assert!(root.path().join("vanilla").is_dir());
        assert_eq!(ws.list_mods().unwrap(), vec![]);
        assert_eq!(ws.enabled_order().unwrap(), Vec::<String>::new());
    }

    #[test]
    fn create_read_round_trip() {
        let (_root, ws) = ws();
        let slug = ws.create_mod(ModManifest::new("Spelling Fixes")).unwrap();
        assert_eq!(slug, "spelling-fixes");

        let pkg = ws.read_mod(&slug).unwrap();
        assert_eq!(pkg.manifest.name, "Spelling Fixes");
        assert!(pkg.changes.is_empty());

        let mods = ws.list_mods().unwrap();
        assert_eq!(mods.len(), 1);
        assert!(!mods[0].enabled);
    }

    #[test]
    fn slug_collision_suffixes() {
        let (_root, ws) = ws();
        let s1 = ws.create_mod(ModManifest::new("typos")).unwrap();
        let s2 = ws.create_mod(ModManifest::new("typos")).unwrap();
        let s3 = ws.create_mod(ModManifest::new("Typos!")).unwrap();
        assert_eq!(s1, "typos");
        assert_eq!(s2, "typos-2");
        assert_eq!(s3, "typos-3");
    }

    #[test]
    fn enable_order_persists() {
        let (_root, ws) = ws();
        let a = ws.create_mod(ModManifest::new("a")).unwrap();
        let b = ws.create_mod(ModManifest::new("b")).unwrap();
        ws.set_enabled(&a, true).unwrap();
        ws.set_enabled(&b, true).unwrap();
        assert_eq!(ws.enabled_order().unwrap(), vec![a.clone(), b.clone()]);
        ws.move_up(&b).unwrap();
        assert_eq!(ws.enabled_order().unwrap(), vec![b.clone(), a.clone()]);
        ws.set_enabled(&b, false).unwrap();
        assert_eq!(ws.enabled_order().unwrap(), vec![a]);
    }

    #[test]
    fn enable_order_filters_deleted_mods() {
        let (_root, ws) = ws();
        let a = ws.create_mod(ModManifest::new("a")).unwrap();
        let b = ws.create_mod(ModManifest::new("b")).unwrap();
        ws.set_enabled(&a, true).unwrap();
        ws.set_enabled(&b, true).unwrap();
        ws.delete_mod(&a).unwrap();
        assert_eq!(ws.enabled_order().unwrap(), vec![b]);
    }

    #[test]
    fn import_export_zip_round_trip() {
        let (root, ws) = ws();
        let slug = ws.create_mod(ModManifest::new("typos")).unwrap();
        let mut pkg = ws.read_mod(&slug).unwrap();
        pkg.changes.push(ChangeAction::new(
            "CharacterInGame/MiscItem.db",
            ChangeOp::FieldDelta {
                record_id: 42,
                field: "name".into(),
                old: Value::Null,
                new: Value::String("Helmet".into()),
            },
        ));
        ws.write_mod(&slug, &pkg).unwrap();

        let zip_path = root.path().join("typos.zip");
        ws.export_zip(&slug, &zip_path).unwrap();
        assert!(zip_path.is_file());

        let new_slug = ws.import_zip(&zip_path).unwrap();
        assert_ne!(new_slug, slug); // collision suffix applied
        let imported = ws.read_mod(&new_slug).unwrap();
        assert_eq!(imported.manifest.name, "typos");
        assert_eq!(imported.changes.len(), 1);
    }

    #[test]
    fn recording_then_apply_round_trip() {
        // Mirrors the GUI flow: create a mod, "record" several field deltas
        // via append_action, then apply to a fake game dir.
        use crate::modding::PatcherRegistry;
        use crate::references::extractor::Extractor;
        use crate::references::misc_item_db::MiscItem;
        use std::io::Cursor;

        let (root, ws) = ws();
        let game = tempdir().unwrap();

        // Seed the game dir with a small MiscItem.db.
        let rel = "CharacterInGame/MiscItem.db";
        let mut data = 1i32.to_le_bytes().to_vec();
        let mut name_buf = [0u8; 30];
        name_buf[..4].copy_from_slice(b"Helt");
        data.extend_from_slice(&name_buf);
        data.extend(vec![0u8; 202]);
        data.extend_from_slice(&15i32.to_le_bytes());
        data.extend(vec![0u8; 20]);
        let game_file = game.path().join(rel);
        fs::create_dir_all(game_file.parent().unwrap()).unwrap();
        fs::write(&game_file, &data).unwrap();

        // "Record" two stringly-typed deltas (as the GUI would).
        let slug = ws.create_mod(ModManifest::new("typos")).unwrap();
        ws.append_action(
            &slug,
            ChangeAction::new(
                rel,
                ChangeOp::FieldDelta {
                    record_id: 0,
                    field: "name".into(),
                    old: Value::String("Helt".into()),
                    new: Value::String("Helmet".into()),
                },
            ),
        )
        .unwrap();
        ws.append_action(
            &slug,
            ChangeAction::new(
                rel,
                ChangeOp::FieldDelta {
                    record_id: 0,
                    field: "base_price".into(),
                    old: Value::String("15".into()),
                    new: Value::String("25".into()),
                },
            ),
        )
        .unwrap();

        ws.set_enabled(&slug, true).unwrap();
        let report = ws
            .apply(game.path(), &PatcherRegistry::with_defaults())
            .unwrap();
        assert_eq!(report.actions_applied, 2);

        let mut c = Cursor::new(fs::read(&game_file).unwrap());
        let recs = MiscItem::parse(&mut c, fs::metadata(&game_file).unwrap().len()).unwrap();
        assert_eq!(recs[0].name, "Helmet");
        assert_eq!(recs[0].base_price, 25);

        let _ = root; // hold tempdir
    }

    #[test]
    fn append_action_grows_changelog() {
        let (_root, ws) = ws();
        let slug = ws.create_mod(ModManifest::new("typos")).unwrap();
        for i in 0..3 {
            ws.append_action(
                &slug,
                ChangeAction::new(
                    "CharacterInGame/MiscItem.db",
                    ChangeOp::FieldDelta {
                        record_id: i,
                        field: "name".into(),
                        old: Value::Null,
                        new: Value::String(format!("v{i}")),
                    },
                ),
            )
            .unwrap();
        }
        let pkg = ws.read_mod(&slug).unwrap();
        assert_eq!(pkg.changes.len(), 3);
    }

    #[test]
    fn list_mods_orders_enabled_first_in_load_order() {
        let (_root, ws) = ws();
        let a = ws.create_mod(ModManifest::new("aaa")).unwrap();
        let b = ws.create_mod(ModManifest::new("bbb")).unwrap();
        let c = ws.create_mod(ModManifest::new("ccc")).unwrap();
        ws.set_enabled(&c, true).unwrap();
        ws.set_enabled(&a, true).unwrap();
        let mods = ws.list_mods().unwrap();
        let slugs: Vec<&str> = mods.iter().map(|m| m.slug.as_str()).collect();
        assert_eq!(slugs, vec![c.as_str(), a.as_str(), b.as_str()]);
        assert!(mods[0].enabled && mods[1].enabled);
        assert!(!mods[2].enabled);
    }
}
