//! The deep modding facade — one small interface over the whole pipeline.
//!
//! Everything that used to be wired up ad-hoc across [`super::change`],
//! [`super::changelog`], [`super::conflicts`], [`super::resolution`],
//! [`super::vanilla`], [`super::apply`], [`super::package`] and
//! [`super::workspace`] is owned here:
//!
//! * **field identity** — one [`FieldKey`] type, carried by [`FieldChange`]
//!   and derived from every [`ChangeOp`];
//! * **change ops** — recording produces [`ChangeAction`]s through the facade;
//! * **detect → pin → apply** — [`Modding::reconcile`] runs the whole state
//!   machine against one consistent snapshot of package + pin state;
//! * **vanilla handling** — snapshots and restores go through the facade's
//!   filesystem port ([`super::fs::Fs`]);
//! * **patcher dispatch** — the [`PatcherRegistry`] is owned by the facade;
//! * **package I/O** — create / import / export / read mods.
//!
//! Callers (CLI, GUI recording glue, hexedit save callback) depend only on
//! this type and on the domain-level [`FieldChange`] event, not on a pile of
//! modules or on UI message types.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::apply::{ApplyReport, RevertReport};
use super::bsdiff;
use super::change::{ChangeAction, ChangeOp, FieldChange};
use super::changelog::ChangeLog;
use super::conflicts::Conflict;
use super::error::{ModdingError, Result};
use super::fs::{Fs, FsHandle, OsFs};
use super::manifest::ModManifest;
use super::package::ModPackage;
use super::registry::PatcherRegistry;
use super::resolution::{FieldKey, ResolutionMap};
use super::vanilla::VanillaStore;
use super::workspace::{InstalledMod, Workspace};

/// Fraction of the full file size at which a bsdiff patch stops being a space
/// win; beyond it we record a whole-file replace instead.
const DELTA_KEEP_THRESHOLD: f64 = 0.7;

/// Choose the most compact whole-file change op for `current` relative to
/// `vanilla`. Shared by the GUI's hex/recording paths so they never have to
/// reach into bsdiff themselves.
pub fn decide_file_op(vanilla: &[u8], current: &[u8]) -> ChangeOp {
    let delta = match bsdiff::make_delta(vanilla, current) {
        Ok(delta) => delta,
        Err(e) => {
            // A failed delta is not fatal — a full replace is always correct —
            // but surface the reason rather than silently discarding it.
            eprintln!("bsdiff delta failed ({e}); recording a full FileReplace");
            return ChangeOp::FileReplace {
                content: current.to_vec(),
            };
        }
    };
    let keep_delta =
        !current.is_empty() && (delta.len() as f64) < (current.len() as f64) * DELTA_KEEP_THRESHOLD;
    if keep_delta {
        ChangeOp::BinaryDelta { patch_bytes: delta }
    } else {
        ChangeOp::FileReplace {
            content: current.to_vec(),
        }
    }
}

/// Outcome of the full detect → pin → apply sequence.
#[derive(Debug, PartialEq)]
pub struct ReconcileReport {
    /// Conflicts detected just before the apply, with pins already pruned.
    pub conflicts: Vec<Conflict>,
    pub apply: ApplyReport,
}

/// The single entry point to mod authoring/packaging/applying.
#[derive(Clone)]
pub struct Modding {
    workspace: Workspace,
    game_dir: PathBuf,
    patchers: PatcherRegistry,
    fs: FsHandle,
}

impl std::fmt::Debug for Modding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Modding")
            .field("workspace", &self.workspace)
            .field("game_dir", &self.game_dir)
            .finish_non_exhaustive()
    }
}

impl Modding {
    /// Open a facade using the real filesystem and all built-in patchers.
    pub fn open(workspace_root: impl Into<PathBuf>, game_dir: impl Into<PathBuf>) -> Result<Self> {
        Self::with_patchers(
            workspace_root,
            game_dir,
            Arc::new(OsFs),
            PatcherRegistry::with_defaults(),
        )
    }

    /// Open a facade with an explicit filesystem port and patcher registry.
    pub fn with_patchers(
        workspace_root: impl Into<PathBuf>,
        game_dir: impl Into<PathBuf>,
        fs: FsHandle,
        patchers: PatcherRegistry,
    ) -> Result<Self> {
        let workspace = Workspace::with_fs(workspace_root.into(), fs.clone())?;
        Ok(Self {
            workspace,
            game_dir: game_dir.into(),
            patchers,
            fs,
        })
    }

    pub fn workspace(&self) -> &Workspace {
        &self.workspace
    }

    pub fn game_dir(&self) -> &Path {
        &self.game_dir
    }

    pub fn fs(&self) -> &dyn Fs {
        self.fs.as_ref()
    }

    pub fn patchers(&self) -> &PatcherRegistry {
        &self.patchers
    }

    // ── Field identity & change ops ──────────────────────────────────────

    /// Build a durable change action from a domain-level field edit event.
    pub fn field_action(
        &self,
        change: FieldChange,
        description: impl Into<String>,
    ) -> ChangeAction {
        change.into_action().with_description(description)
    }

    /// Record one field edit into `slug`'s changelog.
    pub fn record(&self, slug: &str, change: FieldChange) -> Result<()> {
        self.workspace.append_action(slug, change.into_action())
    }

    /// Record several field edits in one read-mutate-write cycle, collapsing
    /// repeated edits to the same [`FieldKey`]. Returns the number collapsed.
    pub fn record_many(
        &self,
        slug: &str,
        changes: impl IntoIterator<Item = FieldChange>,
    ) -> Result<usize> {
        let actions = changes.into_iter().map(FieldChange::into_action).collect();
        self.workspace.append_actions_and_flatten(slug, actions)
    }

    /// Build (but do not persist) a whole-file change for `relative` from the
    /// current bytes, snapshotting vanilla on first contact.
    pub fn file_change(&self, relative: &str, current: &[u8]) -> Result<ChangeAction> {
        let vanilla = self.vanilla_bytes(relative)?.ok_or_else(|| {
            ModdingError::Malformed(format!(
                "vanilla file `{relative}` is not present; cannot diff"
            ))
        })?;
        Ok(ChangeAction::new(
            relative,
            decide_file_op(&vanilla, current),
        ))
    }

    /// Snapshot + record a whole-file change in one call (hex editor saves).
    pub fn record_file(&self, slug: &str, relative: &str, current: &[u8]) -> Result<()> {
        let action = self.file_change(relative, current)?;
        self.append(slug, action)
    }

    /// Append an already-built action. Used by callers that need to describe
    /// the op in a status message before persisting it.
    pub fn append(&self, slug: &str, action: ChangeAction) -> Result<()> {
        self.workspace.append_action(slug, action)
    }

    /// The final changelog for one mod — the object boundary tests assert on.
    pub fn changelog(&self, slug: &str) -> Result<ChangeLog> {
        Ok(self.workspace.read_mod(slug)?.changes)
    }

    // ── detect → pin → apply ─────────────────────────────────────────────

    pub fn conflicts(&self) -> Result<Vec<Conflict>> {
        self.workspace.detect_conflicts()
    }

    pub fn resolutions(&self) -> Result<ResolutionMap> {
        self.workspace.resolutions()
    }

    /// Pin a field to a specific mod's contribution, regardless of load order.
    pub fn pin(&self, key: FieldKey, mod_slug: &str) -> Result<()> {
        self.workspace.pin_resolution(key, mod_slug)
    }

    pub fn unpin(&self, key: &FieldKey) -> Result<()> {
        self.workspace.unpin_resolution(key)
    }

    /// Detect conflicts, then re-apply enabled mods under the same pin map,
    /// and report both. This is the single transaction the GUI library view
    /// and the CLI should call instead of composing those steps themselves.
    pub fn reconcile(&self) -> Result<ReconcileReport> {
        let conflicts = self.conflicts()?;
        let apply = self.apply()?;
        Ok(ReconcileReport { conflicts, apply })
    }

    pub fn apply(&self) -> Result<ApplyReport> {
        self.workspace
            .apply_with_fs(&*self.fs, &self.game_dir, &self.patchers)
    }

    pub fn revert(&self) -> Result<RevertReport> {
        self.workspace.revert_with_fs(&*self.fs, &self.game_dir)
    }

    // ── vanilla handling ─────────────────────────────────────────────────

    /// Ensure a pristine snapshot exists and return the original bytes.
    pub fn vanilla_bytes(&self, relative: &str) -> Result<Option<Vec<u8>>> {
        self.workspace
            .vanilla()
            .ensure_snapshot(&self.game_dir, relative)
    }

    pub fn vanilla(&self) -> &VanillaStore {
        self.workspace.vanilla()
    }

    // ── package I/O ──────────────────────────────────────────────────────

    pub fn list_mods(&self) -> Result<Vec<InstalledMod>> {
        self.workspace.list_mods()
    }

    pub fn read_mod(&self, slug: &str) -> Result<ModPackage> {
        self.workspace.read_mod(slug)
    }

    pub fn create_mod(&self, manifest: ModManifest) -> Result<String> {
        self.workspace.create_mod(manifest)
    }

    pub fn delete_mod(&self, slug: &str) -> Result<()> {
        self.workspace.delete_mod(slug)
    }

    pub fn import_zip(&self, path: &Path) -> Result<String> {
        self.workspace.import_zip(path)
    }

    pub fn export_zip(&self, slug: &str, dst: &Path) -> Result<()> {
        self.workspace.export_zip(slug, dst)
    }

    pub fn set_enabled(&self, slug: &str, enabled: bool) -> Result<()> {
        self.workspace.set_enabled(slug, enabled)
    }

    pub fn remove_action(&self, slug: &str, id: uuid::Uuid) -> Result<()> {
        self.workspace.remove_action(slug, id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modding::fs::MemFs;
    use crate::modding::value::Value;
    use crate::references::extractor::Extractor;
    use crate::references::layout::RecordLayout;
    use crate::references::misc_item_db::MiscItem;
    use std::io::Cursor;

    const REL: &str = "CharacterInGame/MiscItem.db";

    fn misc_bytes(name: &str, price: i32) -> Vec<u8> {
        // Offsets come from the generated layout, not duplicated literals.
        let layout = MiscItem::LAYOUT;
        let mut data = vec![0u8; layout.footprint() as usize];
        data[..layout.header_size as usize].copy_from_slice(&1i32.to_le_bytes());
        let name_def = layout.field("name").unwrap();
        let n = name.len().min(name_def.size as usize - 1);
        let name_start = layout.header_size as usize + name_def.offset as usize;
        data[name_start..name_start + n].copy_from_slice(&name.as_bytes()[..n]);
        let price_def = layout.field("base_price").unwrap();
        let price_start = layout.header_size as usize + price_def.offset as usize;
        data[price_start..price_start + price_def.size as usize]
            .copy_from_slice(&price.to_le_bytes());
        data
    }

    fn names(bytes: &[u8]) -> Vec<String> {
        MiscItem::parse(&mut Cursor::new(bytes), bytes.len() as u64)
            .unwrap()
            .into_iter()
            .map(|r| r.name)
            .collect()
    }

    fn seeded(fs: &Arc<MemFs>, root: &str, game: &str) -> Modding {
        let game = Path::new(game);
        fs.create_dir_all(game).unwrap();
        fs.write(&game.join(REL), &misc_bytes("Helt", 15)).unwrap();
        Modding::with_patchers(
            PathBuf::from(root),
            game.to_path_buf(),
            fs.clone(),
            PatcherRegistry::with_defaults(),
        )
        .unwrap()
    }

    fn field_change(new: &str) -> FieldChange {
        FieldChange::new(
            FieldKey::new(REL, 0, "name"),
            Value::String("Helt".into()),
            Value::String(new.into()),
        )
    }

    #[test]
    fn test_record_then_reconcile_asserts_final_changelog_and_apply() {
        let fs = Arc::new(MemFs::new());
        let game = Path::new("/virtual/game1");
        let m = seeded(&fs, "/virtual/ws1", "/virtual/game1");
        let slug = m.create_mod(ModManifest::new("spelling")).unwrap();
        m.record(&slug, field_change("Helmet")).unwrap();
        m.set_enabled(&slug, true).unwrap();

        // Assert the final ChangeLog, not a pending-map check.
        let log = m.changelog(&slug).unwrap();
        assert_eq!(log.len(), 1);
        match &log.actions()[0].op {
            ChangeOp::FieldDelta {
                record_id,
                field,
                old,
                new,
            } => {
                assert_eq!(*record_id, 0);
                assert_eq!(field, "name");
                assert_eq!(old, &Value::String("Helt".into()));
                assert_eq!(new, &Value::String("Helmet".into()));
            }
            other => panic!("wrong op: {other:?}"),
        }
        // FieldKey identity is shared: the op derives the very key we recorded.
        assert_eq!(
            log.actions()[0].op.field_key(REL),
            Some(FieldKey::new(REL, 0, "name"))
        );

        let report = m.reconcile().unwrap();
        assert!(report.conflicts.is_empty());
        assert_eq!(report.apply.actions_applied, 1);
        assert_eq!(names(&fs.read(&game.join(REL)).unwrap()), vec!["Helmet"]);

        // Revert through the port restores vanilla.
        m.revert().unwrap();
        assert_eq!(names(&fs.read(&game.join(REL)).unwrap()), vec!["Helt"]);
    }

    #[test]
    fn test_detect_pin_apply_agree_on_same_field() {
        let fs = Arc::new(MemFs::new());
        let game = Path::new("/virtual/game2");
        let m = seeded(&fs, "/virtual/ws2", "/virtual/game2");
        let a = m.create_mod(ModManifest::new("a")).unwrap();
        let b = m.create_mod(ModManifest::new("b")).unwrap();
        for (slug, new) in [(&a, "Helmet"), (&b, "Hat")] {
            m.record(slug, field_change(new)).unwrap();
            m.set_enabled(slug, true).unwrap();
        }

        // Load order [a, b]: without a pin b wins.
        let conflicts = m.conflicts().unwrap();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].winner(), b);
        m.apply().unwrap();
        assert_eq!(names(&fs.read(&game.join(REL)).unwrap()), vec!["Hat"]);

        // Reordering flips the winner and the applied bytes together.
        m.workspace().move_up(&b).unwrap();
        let report = m.reconcile().unwrap();
        assert_eq!(report.conflicts[0].winner(), a);
        assert_eq!(names(&fs.read(&game.join(REL)).unwrap()), vec!["Helmet"]);

        // Pin to a: both the conflict view and the apply engine must agree.
        m.workspace().move_down(&b).unwrap();
        m.pin(FieldKey::new(REL, 0, "name"), &a).unwrap();
        let report = m.reconcile().unwrap();
        assert_eq!(report.conflicts[0].winner(), a);
        assert_eq!(names(&fs.read(&game.join(REL)).unwrap()), vec!["Helmet"]);
        assert_eq!(m.resolutions().unwrap().len(), 1);

        // Disabling the pinned mod drops its pin on the next reconcile.
        m.set_enabled(&a, false).unwrap();
        m.reconcile().unwrap();
        assert!(m.resolutions().unwrap().is_empty());
        assert_eq!(names(&fs.read(&game.join(REL)).unwrap()), vec!["Hat"]);
    }

    #[test]
    fn test_record_many_flattens_to_final_changelog() {
        let fs = Arc::new(MemFs::new());
        let m = seeded(&fs, "/virtual/ws3", "/virtual/game3");
        let slug = m.create_mod(ModManifest::new("typing")).unwrap();
        let removed = m
            .record_many(
                &slug,
                [
                    FieldChange::new(
                        FieldKey::new(REL, 0, "name"),
                        Value::String("Helt".into()),
                        Value::String("Hel".into()),
                    ),
                    FieldChange::new(
                        FieldKey::new(REL, 0, "name"),
                        Value::String("Hel".into()),
                        Value::String("Helmet".into()),
                    ),
                ],
            )
            .unwrap();
        assert_eq!(removed, 1);
        let log = m.changelog(&slug).unwrap();
        assert_eq!(log.len(), 1);
        match &log.actions()[0].op {
            ChangeOp::FieldDelta { old, new, .. } => {
                assert_eq!(old, &Value::String("Helt".into()));
                assert_eq!(new, &Value::String("Helmet".into()));
            }
            other => panic!("wrong op: {other:?}"),
        }
    }

    #[test]
    fn test_file_change_records_full_replace_when_delta_is_not_worth_it() {
        let fs = Arc::new(MemFs::new());
        let m = Modding::with_patchers(
            PathBuf::from("/virtual/ws4"),
            PathBuf::from("/virtual/game4"),
            fs.clone(),
            PatcherRegistry::with_defaults(),
        )
        .unwrap();
        let vanilla: Vec<u8> = (0u8..64).collect();
        let current: Vec<u8> = (0u8..64).map(|b| 255 - b).collect();
        fs.write(Path::new("/virtual/game4/Sprite/new.spr"), &vanilla)
            .unwrap();

        let action = m.file_change("Sprite/new.spr", &current).unwrap();
        assert!(matches!(action.op, ChangeOp::FileReplace { .. }));
    }

    #[test]
    fn test_unflattened_changelog_winner_matches_applied_bytes() {
        // A changelog written by an older tool can contain several edits to the
        // same field. Write one straight into the package directory (bypassing
        // the normalizing append path) and assert that conflict detection and
        // apply still agree about the winning value.
        let fs = Arc::new(MemFs::new());
        let game = Path::new("/virtual/game5");
        let m = seeded(&fs, "/virtual/ws5", "/virtual/game5");
        let a = m.create_mod(ModManifest::new("a")).unwrap();
        let b = m.create_mod(ModManifest::new("b")).unwrap();
        m.record(&b, field_change("Hat")).unwrap();

        let unflattened = vec![
            ChangeAction::new(
                REL,
                ChangeOp::FieldDelta {
                    record_id: 0,
                    field: "name".into(),
                    old: Value::String("Helt".into()),
                    new: Value::String("Hel".into()),
                },
            ),
            ChangeAction::new(
                REL,
                ChangeOp::FieldDelta {
                    record_id: 0,
                    field: "name".into(),
                    old: Value::String("Hel".into()),
                    new: Value::String("Helmet".into()),
                },
            ),
        ];
        fs.write(
            Path::new("/virtual/ws5/mods/a/changes.json"),
            &serde_json::to_vec_pretty(&unflattened).unwrap(),
        )
        .unwrap();
        assert_eq!(m.changelog(&a).unwrap().len(), 2);

        m.set_enabled(&a, true).unwrap();
        m.set_enabled(&b, true).unwrap();

        // Last mod in load order wins, and its value is what gets written.
        let report = m.reconcile().unwrap();
        assert_eq!(report.conflicts[0].winner(), b);
        assert_eq!(names(&fs.read(&game.join(REL)).unwrap()), vec!["Hat"]);

        // Pin to a: a's *last* unflattened edit (Helmet) becomes the winner,
        // and those exact bytes land in the game file.
        m.pin(FieldKey::new(REL, 0, "name"), &a).unwrap();
        let report = m.reconcile().unwrap();
        assert_eq!(report.conflicts[0].winner(), a);
        assert_eq!(
            report.conflicts[0].participants[0].field_new,
            Some(Value::String("Helmet".into()))
        );
        assert_eq!(names(&fs.read(&game.join(REL)).unwrap()), vec!["Helmet"]);
    }

    #[test]
    fn test_mod_lifecycle_boundary_create_enable_export_import_disable_delete() {
        // The whole lifecycle runs against MemFs: no real directory is made.
        let fs = Arc::new(MemFs::new());
        let m = seeded(&fs, "/virtual/ws6", "/virtual/game6");
        let a = m.create_mod(ModManifest::new("alpha")).unwrap();
        assert_eq!(a, "alpha");
        assert!(fs.is_dir(Path::new("/virtual/ws6/mods/alpha")));

        m.set_enabled(&a, true).unwrap();
        assert_eq!(m.workspace().enabled_order().unwrap(), vec![a.clone()]);

        let zip = Path::new("/virtual/exports/alpha.zip");
        m.export_zip(&a, zip).unwrap();
        assert!(fs.is_file(zip));

        let copy = m.import_zip(zip).unwrap();
        assert_eq!(copy, "alpha-2");
        assert_eq!(m.list_mods().unwrap().len(), 2);
        m.set_enabled(&copy, true).unwrap();
        assert_eq!(
            m.workspace().enabled_order().unwrap(),
            vec![a.clone(), copy.clone()]
        );

        m.set_enabled(&copy, false).unwrap();
        assert_eq!(m.workspace().enabled_order().unwrap(), vec![a.clone()]);

        m.delete_mod(&copy).unwrap();
        assert!(!fs.exists(Path::new("/virtual/ws6/mods/alpha-2")));
        assert_eq!(m.list_mods().unwrap().len(), 1);
    }
}
