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
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::apply::{
    ApplyReport, ModEntry, RevertReport, apply_all_with_fs, revert_to_vanilla_with_fs,
};
use super::error::{ModdingError, Result};
use super::fs::{Fs, FsHandle, OsFs};
use super::manifest::ModManifest;
use super::package::{self, ModPackage};
use super::registry::PatcherRegistry;
use super::resolution::{FieldKey, ResolutionMap};
use super::vanilla::VanillaStore;

const MODS_DIR: &str = "mods";
const VANILLA_DIR: &str = "vanilla";
const ENABLED_FILE: &str = "enabled.json";
const RESOLUTIONS_FILE: &str = "resolutions.json";

/// Lightweight summary of one installed mod, suitable for the Library list.
#[derive(Debug, Clone, PartialEq)]
pub struct InstalledMod {
    pub slug: String,
    pub manifest: ModManifest,
    pub change_count: usize,
    pub enabled: bool,
}

/// What a recording append did to the changelog it wrote.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AppendOutcome {
    /// Actions removed by field-delta flattening.
    pub collapsed: usize,
    /// Oldest actions dropped at [`super::HISTORY_CAP`].
    pub pruned: usize,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct EnabledFile {
    /// Slugs in load order (first = applied first).
    enabled: Vec<String>,
}

/// Open or create a mod workspace at `root`.
#[derive(Clone)]
pub struct Workspace {
    root: PathBuf,
    vanilla: VanillaStore,
    fs: FsHandle,
}

impl std::fmt::Debug for Workspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Workspace")
            .field("root", &self.root)
            .finish_non_exhaustive()
    }
}

impl Workspace {
    /// Open (and create-on-first-use) a workspace rooted at `root`.
    pub fn open(root: PathBuf) -> Result<Self> {
        Self::with_fs(root, Arc::new(OsFs))
    }

    /// Open a workspace whose *entire* storage — layout directories, state
    /// files, mod packages, vanilla mirror — goes through `fs`. Production
    /// passes [`OsFs`]; tests pass an in-memory stand-in.
    pub fn with_fs(root: PathBuf, fs: FsHandle) -> Result<Self> {
        fs.create_dir_all(&root)?;
        fs.create_dir_all(&root.join(MODS_DIR))?;
        let vanilla_root = root.join(VANILLA_DIR);
        fs.create_dir_all(&vanilla_root)?;
        Ok(Self {
            vanilla: VanillaStore::with_fs(vanilla_root, fs.clone()),
            root,
            fs,
        })
    }

    /// The filesystem port used for game/vanilla I/O.
    pub fn fs(&self) -> &dyn Fs {
        self.fs.as_ref()
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

    fn resolutions_path(&self) -> PathBuf {
        self.root.join(RESOLUTIONS_FILE)
    }

    /// Read the persisted per-field resolutions (pins). Returns an empty map
    /// if the file is missing.
    pub fn resolutions(&self) -> Result<ResolutionMap> {
        let path = self.resolutions_path();
        if !self.fs.is_file(&path) {
            return Ok(ResolutionMap::new());
        }
        let raw = self.fs.read(&path)?;
        Ok(serde_json::from_slice(&raw)?)
    }

    fn write_resolutions(&self, map: &ResolutionMap) -> Result<()> {
        self.fs
            .write(&self.resolutions_path(), &serde_json::to_vec_pretty(map)?)?;
        Ok(())
    }

    /// Pin a field to a specific mod's value, regardless of load order.
    /// Subsequent applies will use that mod's contribution to this key.
    pub fn pin_resolution(&self, key: FieldKey, mod_slug: &str) -> Result<()> {
        validate_slug(mod_slug)?;
        let mut map = self.resolutions()?;
        map.pin(key, mod_slug);
        self.write_resolutions(&map)
    }

    /// Drop the pin for `key`, falling back to load-order resolution.
    pub fn unpin_resolution(&self, key: &FieldKey) -> Result<()> {
        let mut map = self.resolutions()?;
        map.unpin(key);
        self.write_resolutions(&map)
    }

    fn mod_dir(&self, slug: &str) -> Result<PathBuf> {
        validate_slug(slug)?;
        Ok(self.mods_root().join(slug))
    }

    /// Read the persisted load order. Slugs no longer present on disk are
    /// dropped silently.
    pub fn enabled_order(&self) -> Result<Vec<String>> {
        let path = self.enabled_path();
        if !self.fs.is_file(&path) {
            return Ok(Vec::new());
        }
        let bytes = self.fs.read(&path)?;
        let raw: EnabledFile = serde_json::from_slice(&bytes)?;
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
                return Err(ModdingError::Malformed(format!("unknown mod slug: {slug}")));
            }
        }
        let raw = EnabledFile { enabled: order };
        self.fs
            .write(&self.enabled_path(), &serde_json::to_vec_pretty(&raw)?)?;
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
        if let Some(idx) = order.iter().position(|s| s == slug)
            && idx > 0
        {
            order.swap(idx - 1, idx);
            self.set_enabled_order(order)?;
        }
        Ok(())
    }

    /// Move a mod one position later in load order.
    pub fn move_down(&self, slug: &str) -> Result<()> {
        let mut order = self.enabled_order()?;
        if let Some(idx) = order.iter().position(|s| s == slug)
            && idx + 1 < order.len()
        {
            order.swap(idx, idx + 1);
            self.set_enabled_order(order)?;
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
        if !self.fs.is_dir(&dir) {
            return Ok(None);
        }
        let pkg = package::read_dir_with_fs(&*self.fs, &dir)?;
        Ok(Some(InstalledMod {
            slug: slug.to_owned(),
            change_count: pkg.changes.len(),
            manifest: pkg.manifest,
            enabled: false, // caller fills this in
        }))
    }

    fn list_slugs(&self) -> Result<Vec<String>> {
        let mods = self.mods_root();
        if !self.fs.is_dir(&mods) {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for entry in self.fs.read_dir(&mods)? {
            if !self.fs.is_dir(&entry) {
                continue;
            }
            if let Some(name) = entry.file_name().and_then(|n| n.to_str()) {
                out.push(name.to_owned());
            }
        }
        Ok(out)
    }

    /// Read a mod package (manifest + full ChangeLog with blobs) by slug.
    pub fn read_mod(&self, slug: &str) -> Result<ModPackage> {
        package::read_dir_with_fs(&*self.fs, &self.mod_dir(slug)?)
    }

    /// Persist a mod package to its directory. The slug is taken as-is; use
    /// [`Self::create_mod`] if you need slug derivation.
    pub fn write_mod(&self, slug: &str, package: &ModPackage) -> Result<()> {
        package::write_dir_with_fs(&*self.fs, &self.mod_dir(slug)?, package)
    }

    /// Create a brand-new mod with the given manifest. Returns the assigned
    /// slug (derived from the manifest name with collision suffixing).
    pub fn create_mod(&self, manifest: ModManifest) -> Result<String> {
        let slug = self.allocate_slug(&manifest.name)?;
        let pkg = ModPackage::new(manifest, super::ChangeLog::new());
        self.write_mod(&slug, &pkg)?;
        Ok(slug)
    }

    /// Remove a [`ChangeAction`](super::change::ChangeAction) from the named
    /// mod by its [`Uuid`](uuid::Uuid). Returns an error if the action is not
    /// found. After removal, [`ChangeLog::flatten_field_deltas`] is called to
    /// coalesce any remaining field deltas that may now be adjacent.
    pub fn remove_action(&self, slug: &str, action_id: uuid::Uuid) -> Result<()> {
        let mut pkg = self.read_mod(slug)?;
        if !pkg.changes.remove_by_id(action_id) {
            return Err(ModdingError::Malformed(format!(
                "action {action_id} not found in mod {slug}"
            )));
        }
        self.write_mod(slug, &pkg)
    }

    /// Append one [`ChangeAction`](super::change::ChangeAction) to the named
    /// mod, in load-mutate-save fashion. Used by recording mode.
    ///
    /// This is the single-action form of [`Self::append_actions_and_flatten`]:
    /// it applies exactly the same normalization, so no append path can leave
    /// an unnormalized changelog behind.
    pub fn append_action(&self, slug: &str, action: super::change::ChangeAction) -> Result<()> {
        self.append_action_reporting(slug, action)?;
        Ok(())
    }

    /// Single-action form of [`Self::append_actions_reporting`], returning the
    /// number of oldest actions dropped at the cap.
    pub fn append_action_reporting(
        &self,
        slug: &str,
        action: super::change::ChangeAction,
    ) -> Result<usize> {
        Ok(self.append_actions_reporting(slug, vec![action])?.pruned)
    }

    /// Append actions in one read-mutate-write cycle, normalize the result by
    /// collapsing repeated `FieldDelta` entries on the same
    /// `(file, record_id, field)` (earliest `old`, latest `new`; no-ops drop),
    /// and report both the collapsed and cap-pruned counts. Every append path
    /// funnels through here; the flattened action set is what gets persisted.
    pub fn append_actions_reporting(
        &self,
        slug: &str,
        actions: Vec<super::change::ChangeAction>,
    ) -> Result<AppendOutcome> {
        let mut pkg = self.read_mod(slug)?;
        let mut pruned = 0;
        for a in actions {
            pruned += pkg.changes.push(a);
        }
        let collapsed = pkg.changes.flatten_field_deltas();
        self.write_mod(slug, &pkg)?;
        Ok(AppendOutcome { collapsed, pruned })
    }

    /// Append actions and return only the number removed by flattening.
    pub fn append_actions_and_flatten(
        &self,
        slug: &str,
        actions: Vec<super::change::ChangeAction>,
    ) -> Result<usize> {
        Ok(self.append_actions_reporting(slug, actions)?.collapsed)
    }

    pub fn delete_mod(&self, slug: &str) -> Result<()> {
        let dir = self.mod_dir(slug)?;
        if self.fs.is_dir(&dir) {
            self.fs.remove_dir_all(&dir)?;
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
        let meta = self.fs.metadata(zip_path)?;
        if !meta.is_file {
            return Err(ModdingError::Malformed(format!(
                "not a zip file: {}",
                zip_path.display()
            )));
        }
        let pkg = package::read_zip(Cursor::new(self.fs.read(zip_path)?))?;
        let slug = self.allocate_slug(&pkg.manifest.name)?;
        self.write_mod(&slug, &pkg)?;
        Ok(slug)
    }

    /// Export an installed mod to `dst` (a `.zip` file path).
    pub fn export_zip(&self, slug: &str, dst: &Path) -> Result<()> {
        let pkg = self.read_mod(slug)?;
        let mut sink = Cursor::new(Vec::new());
        package::write_zip(&mut sink, &pkg)?;
        self.fs.write(dst, &sink.into_inner())
    }

    /// Load the enabled mods in load order together with their resolution map.
    ///
    /// Shared by [`Self::apply`] and [`Self::detect_conflicts`]: both stages of
    /// the detect → pin → apply state machine must see the *same* packages and
    /// the *same* (pruned) pins, otherwise the conflict UI can disagree with
    /// what actually gets written.
    fn enabled_state(&self) -> Result<(Vec<String>, Vec<ModPackage>, ResolutionMap)> {
        let order = self.enabled_order()?;
        let mut resolutions = self.resolutions()?;
        if resolutions.prune_to(&order) > 0 {
            self.write_resolutions(&resolutions)?;
        }
        let packages: Vec<ModPackage> = order
            .iter()
            .map(|slug| self.read_mod(slug))
            .collect::<Result<Vec<_>>>()?;
        Ok((order, packages, resolutions))
    }

    /// Borrow the loaded packages as apply-engine entries, in load order.
    fn entries<'a>(order: &'a [String], packages: &'a [ModPackage]) -> Vec<ModEntry<'a>> {
        order
            .iter()
            .zip(packages.iter())
            .map(|(slug, pkg)| ModEntry {
                mod_id: slug.as_str(),
                changes: &pkg.changes,
            })
            .collect()
    }

    /// Apply all enabled mods to `game_dir`. Honors any persisted pins —
    /// stale pins (pointing at disabled mods) are auto-pruned and persisted
    /// before the apply runs.
    pub fn apply(&self, game_dir: &Path, registry: &PatcherRegistry) -> Result<ApplyReport> {
        self.apply_with_fs(&*self.fs, game_dir, registry)
    }

    /// Apply through an explicit filesystem port (used by the facade).
    pub fn apply_with_fs(
        &self,
        fs: &dyn Fs,
        game_dir: &Path,
        registry: &PatcherRegistry,
    ) -> Result<ApplyReport> {
        let (order, packages, resolutions) = self.enabled_state()?;
        let mods = Self::entries(&order, &packages);
        apply_all_with_fs(fs, &mods, game_dir, &self.vanilla, registry, &resolutions)
    }

    /// Restore every snapshotted file back to vanilla state.
    pub fn revert(&self, game_dir: &Path) -> Result<RevertReport> {
        self.revert_with_fs(&*self.fs, game_dir)
    }

    /// Revert through an explicit filesystem port (used by the facade).
    pub fn revert_with_fs(&self, fs: &dyn Fs, game_dir: &Path) -> Result<RevertReport> {
        revert_to_vanilla_with_fs(fs, game_dir, &self.vanilla)
    }

    /// Detect conflicts across the enabled mods, in load order. Uses the same
    /// pruned pin map as [`Self::apply`] so the reported winner is the value
    /// that will actually be written.
    pub fn detect_conflicts(&self) -> Result<Vec<super::conflicts::Conflict>> {
        let (order, packages, resolutions) = self.enabled_state()?;
        let mods = Self::entries(&order, &packages);
        Ok(super::conflicts::detect_conflicts_with(&mods, &resolutions))
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
    use crate::modding::fs::MemFs;
    use crate::modding::test_helpers::misc_bytes;
    use crate::modding::value::Value;
    use crate::references::extractor::Extractor;
    use crate::references::misc_item_db::MiscItem;
    use std::io::Cursor;

    const GAME: &str = "/virtual/game";

    fn ws_at(fs: &Arc<MemFs>, root: &str) -> Workspace {
        Workspace::with_fs(PathBuf::from(root), fs.clone()).unwrap()
    }

    fn setup(root: &str) -> (Arc<MemFs>, Workspace) {
        let fs = Arc::new(MemFs::new());
        let ws = ws_at(&fs, root);
        (fs, ws)
    }

    fn field_action(rel: &str, rec: u32, field: &str, new: Value) -> ChangeAction {
        ChangeAction::new(
            rel,
            ChangeOp::FieldDelta {
                record_id: rec,
                field: field.into(),
                old: Value::Null,
                new,
            },
        )
    }

    #[test]
    fn open_creates_subdirs_in_the_port() {
        let (fs, ws) = setup("/ws-open");
        assert!(fs.is_dir(Path::new("/ws-open/mods")));
        assert!(fs.is_dir(Path::new("/ws-open/vanilla")));
        assert_eq!(ws.list_mods().unwrap(), vec![]);
        assert_eq!(ws.enabled_order().unwrap(), Vec::<String>::new());
    }

    #[test]
    fn create_read_round_trip() {
        let (_fs, ws) = setup("/ws-create");
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
        let (_fs, ws) = setup("/ws-slugs");
        let s1 = ws.create_mod(ModManifest::new("typos")).unwrap();
        let s2 = ws.create_mod(ModManifest::new("typos")).unwrap();
        let s3 = ws.create_mod(ModManifest::new("Typos!")).unwrap();
        assert_eq!(s1, "typos");
        assert_eq!(s2, "typos-2");
        assert_eq!(s3, "typos-3");
    }

    #[test]
    fn enable_order_persists() {
        let (_fs, ws) = setup("/ws-order");
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
        let (_fs, ws) = setup("/ws-deleted");
        let a = ws.create_mod(ModManifest::new("a")).unwrap();
        let b = ws.create_mod(ModManifest::new("b")).unwrap();
        ws.set_enabled(&a, true).unwrap();
        ws.set_enabled(&b, true).unwrap();
        ws.delete_mod(&a).unwrap();
        assert_eq!(ws.enabled_order().unwrap(), vec![b]);
    }

    #[test]
    fn import_export_zip_round_trip() {
        let (fs, ws) = setup("/ws-zip");
        let slug = ws.create_mod(ModManifest::new("typos")).unwrap();
        let mut pkg = ws.read_mod(&slug).unwrap();
        pkg.changes.push(field_action(
            "CharacterInGame/MiscItem.db",
            42,
            "name",
            Value::String("Helmet".into()),
        ));
        ws.write_mod(&slug, &pkg).unwrap();

        let zip_path = Path::new("/exports/typos.zip");
        ws.export_zip(&slug, zip_path).unwrap();
        assert!(fs.is_file(zip_path));

        let new_slug = ws.import_zip(zip_path).unwrap();
        assert_ne!(new_slug, slug); // collision suffix applied
        let imported = ws.read_mod(&new_slug).unwrap();
        assert_eq!(imported.manifest.name, "typos");
        assert_eq!(imported.changes.len(), 1);
    }

    #[test]
    fn create_delete_mod_removes_all_files_and_pins() {
        let (fs, ws) = setup("/ws-delete-mod");
        let slug = ws.create_mod(ModManifest::new("delete-me")).unwrap();
        let dir = PathBuf::from("/ws-delete-mod/mods").join(&slug);
        assert!(fs.is_dir(&dir));
        assert_eq!(fs.read_dir(&dir).unwrap().len(), 2); // manifest + changes

        ws.set_enabled(&slug, true).unwrap();
        ws.delete_mod(&slug).unwrap();
        assert!(!fs.exists(&dir));
        assert!(ws.enabled_order().unwrap().is_empty());
        assert!(ws.read_mod(&slug).is_err());
    }

    #[test]
    fn recording_then_apply_round_trip() {
        // Mirrors the GUI flow, with every byte — game files included — in the
        // in-memory filesystem.
        let (fs, ws) = setup("/ws-record");
        let rel = "CharacterInGame/MiscItem.db";
        let game = Path::new(GAME);
        fs.create_dir_all(game).unwrap();
        fs.write(&game.join(rel), &misc_bytes("Helt", 15)).unwrap();

        let slug = ws.create_mod(ModManifest::new("typos")).unwrap();
        ws.append_action(
            &slug,
            field_action(rel, 0, "name", Value::String("Helmet".into())),
        )
        .unwrap();
        ws.append_action(&slug, field_action(rel, 0, "base_price", Value::I64(25)))
            .unwrap();

        ws.set_enabled(&slug, true).unwrap();
        let report = ws.apply(game, &PatcherRegistry::with_defaults()).unwrap();
        assert_eq!(report.actions_applied, 2);

        let bytes = fs.read(&game.join(rel)).unwrap();
        let len = bytes.len() as u64;
        let recs = MiscItem::parse(&mut Cursor::new(bytes), len).unwrap();
        assert_eq!(recs[0].name, "Helmet");
        assert_eq!(recs[0].base_price, 25);
    }

    #[test]
    fn append_action_grows_changelog() {
        let (_fs, ws) = setup("/ws-append");
        let slug = ws.create_mod(ModManifest::new("typos")).unwrap();
        for i in 0..3 {
            ws.append_action(
                &slug,
                field_action(
                    "CharacterInGame/MiscItem.db",
                    i,
                    "name",
                    Value::String(format!("v{i}")),
                ),
            )
            .unwrap();
        }
        assert_eq!(ws.read_mod(&slug).unwrap().changes.len(), 3);
    }

    #[test]
    fn appending_same_field_twice_flattens_through_one_contract() {
        let (_fs, ws) = setup("/ws-flatten");
        let slug = ws.create_mod(ModManifest::new("typos")).unwrap();
        let action = |old: &str, new: &str| {
            field_action("MiscItem.db", 0, "name", Value::String(new.into()))
                .with_description(format!("{old} -> {new}"))
        };
        // Single appends and bulk appends now share the same normalization.
        ws.append_action(&slug, action("a", "b")).unwrap();
        ws.append_action(&slug, action("b", "c")).unwrap();
        ws.append_actions_and_flatten(&slug, vec![action("c", "d")])
            .unwrap();
        let log = ws.read_mod(&slug).unwrap().changes;
        assert_eq!(log.len(), 1);
        assert_eq!(
            log.actions()[0].op.field_key("MiscItem.db").unwrap().field,
            "name"
        );
    }

    #[test]
    fn append_actions_reporting_surfaces_cap_prune() {
        let (_fs, ws) = setup("/ws-cap-bulk");
        let slug = ws.create_mod(ModManifest::new("long-history")).unwrap();
        let actions: Vec<_> = (0..=crate::modding::HISTORY_CAP)
            .map(|i| {
                field_action(
                    "MiscItem.db",
                    i as u32,
                    "name",
                    Value::String(format!("v{i}")),
                )
            })
            .collect();

        let outcome = ws.append_actions_reporting(&slug, actions).unwrap();
        assert_eq!(outcome.pruned, 1);
        assert_eq!(outcome.collapsed, 0);

        let log = ws.read_mod(&slug).unwrap().changes;
        assert_eq!(log.len(), crate::modding::HISTORY_CAP);
        let ChangeOp::FieldDelta {
            record_id: first, ..
        } = &log.actions()[0].op
        else {
            unreachable!()
        };
        let ChangeOp::FieldDelta {
            record_id: last, ..
        } = &log.actions()[crate::modding::HISTORY_CAP - 1].op
        else {
            unreachable!()
        };
        assert_eq!(*first, 1);
        assert_eq!(*last, crate::modding::HISTORY_CAP as u32);
    }

    #[test]
    fn append_action_reporting_surfaces_cap_prune() {
        let (_fs, ws) = setup("/ws-cap-single");
        let slug = ws.create_mod(ModManifest::new("long-history")).unwrap();
        for i in 0..crate::modding::HISTORY_CAP {
            assert_eq!(
                ws.append_action_reporting(
                    &slug,
                    field_action(
                        "MiscItem.db",
                        i as u32,
                        "name",
                        Value::String(format!("v{i}")),
                    ),
                )
                .unwrap(),
                0
            );
        }
        assert_eq!(
            ws.append_action_reporting(
                &slug,
                field_action(
                    "MiscItem.db",
                    crate::modding::HISTORY_CAP as u32,
                    "name",
                    Value::String("last".into()),
                ),
            )
            .unwrap(),
            1
        );
    }

    #[test]
    fn pin_unpin_persists_across_reopen() {
        let (fs, ws) = setup("/ws-pins");
        let a = ws.create_mod(ModManifest::new("a")).unwrap();
        let key = FieldKey {
            file_path: "MiscItem.db".into(),
            record_id: 0,
            field: "name".into(),
        };
        ws.pin_resolution(key.clone(), &a).unwrap();
        assert_eq!(ws.resolutions().unwrap().winner(&key), Some(a.as_str()));

        // Reopen against the same port — the pin survives.
        let ws2 = ws_at(&fs, "/ws-pins");
        assert_eq!(ws2.resolutions().unwrap().winner(&key), Some(a.as_str()));

        ws2.unpin_resolution(&key).unwrap();
        assert!(ws2.resolutions().unwrap().winner(&key).is_none());
    }

    #[test]
    fn apply_prunes_pins_to_disabled_mods() {
        let (_fs, ws) = setup("/ws-prune");
        let a = ws.create_mod(ModManifest::new("a")).unwrap();
        let key = FieldKey {
            file_path: "MiscItem.db".into(),
            record_id: 0,
            field: "name".into(),
        };
        ws.pin_resolution(key.clone(), &a).unwrap();
        // a is not enabled — apply should drop the pin.
        fs_create_game(&ws);
        ws.apply(Path::new(GAME), &PatcherRegistry::new()).unwrap();
        assert!(ws.resolutions().unwrap().winner(&key).is_none());
    }

    fn fs_create_game(ws: &Workspace) {
        let game = Path::new(GAME);
        if !ws.fs().is_dir(game) {
            ws.fs().create_dir_all(game).unwrap();
        }
    }

    #[test]
    fn list_mods_orders_enabled_first_in_load_order() {
        let (_fs, ws) = setup("/ws-list");
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
