//! Read and write `mod.zip` packages.
//!
//! Layout:
//!
//! ```text
//! mod.zip/
//!   manifest.json          ModManifest
//!   changes.json           Vec<ChangeAction>     (without inline blobs)
//!   patches/<uuid>.bin     qbsdiff patch bytes for BinaryDelta actions
//!   files/<uuid>.bin       full content for FileReplace / FileAdd
//! ```
//!
//! The `patches/` and `files/` directories only appear when relevant actions
//! are present. Field-only mods produce a two-file zip.

use std::fs;
use std::io::{Read, Seek, Write};
use std::path::Path;

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use super::change::{BlobKind, ChangeAction};
use super::changelog::ChangeLog;
use super::error::{ModdingError, Result};
use super::manifest::{ModManifest, MANIFEST_VERSION};

pub const MANIFEST_ENTRY: &str = "manifest.json";
pub const CHANGES_ENTRY: &str = "changes.json";

/// In-memory representation of a complete mod package.
#[derive(Debug, Clone, PartialEq)]
pub struct ModPackage {
    pub manifest: ModManifest,
    pub changes: ChangeLog,
}

impl ModPackage {
    pub fn new(manifest: ModManifest, changes: ChangeLog) -> Self {
        Self { manifest, changes }
    }
}

/// Write a mod package to any [`Write`] + [`Seek`] sink (typically a `File`
/// or `Cursor<Vec<u8>>`).
pub fn write_zip<W: Write + Seek>(sink: W, package: &ModPackage) -> Result<()> {
    let mut zip = ZipWriter::new(sink);
    let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    // manifest.json
    zip.start_file(MANIFEST_ENTRY, opts)?;
    serde_json::to_writer_pretty(&mut zip, &package.manifest)?;

    // changes.json — note: blobs are #[serde(skip)] so this is index-only.
    zip.start_file(CHANGES_ENTRY, opts)?;
    serde_json::to_writer_pretty(&mut zip, package.changes.actions())?;

    // Out-of-line blobs.
    for action in package.changes.actions() {
        if let Some((kind, bytes)) = action.op.out_of_line_bytes() {
            let entry_name = blob_entry_name(kind, action);
            zip.start_file(&entry_name, opts)?;
            zip.write_all(bytes)?;
        }
    }

    zip.finish()?;
    Ok(())
}

/// Read a mod package from any [`Read`] + [`Seek`] source.
pub fn read_zip<R: Read + Seek>(source: R) -> Result<ModPackage> {
    let mut zip = ZipArchive::new(source)?;

    let manifest: ModManifest = {
        let mut entry = zip
            .by_name(MANIFEST_ENTRY)
            .map_err(|_| ModdingError::MissingEntry(MANIFEST_ENTRY.into()))?;
        serde_json::from_reader(&mut entry)?
    };
    if manifest.manifest_version > MANIFEST_VERSION {
        return Err(ModdingError::UnsupportedManifestVersion(
            manifest.manifest_version,
        ));
    }

    let mut actions: Vec<ChangeAction> = {
        let mut entry = zip
            .by_name(CHANGES_ENTRY)
            .map_err(|_| ModdingError::MissingEntry(CHANGES_ENTRY.into()))?;
        serde_json::from_reader(&mut entry)?
    };

    // Re-attach out-of-line blobs.
    for action in actions.iter_mut() {
        let Some((kind, _)) = action.op.out_of_line_bytes() else {
            continue;
        };
        let entry_name = blob_entry_name(kind, action);
        let bytes = match zip.by_name(&entry_name) {
            Ok(mut entry) => {
                let mut buf = Vec::with_capacity(entry.size() as usize);
                entry.read_to_end(&mut buf)?;
                buf
            }
            Err(_) => {
                return Err(ModdingError::MissingEntry(entry_name));
            }
        };
        action.op.attach_blob(bytes);
    }

    Ok(ModPackage::new(manifest, ChangeLog::from_actions(actions)))
}

fn blob_entry_name(kind: BlobKind, action: &ChangeAction) -> String {
    format!("{}/{}.bin", kind.dir_name(), action.id)
}

/// Write a mod package as a directory mirroring the zip layout.
///
/// Used by [`Workspace`](super::workspace::Workspace) for the live, editable
/// copy of each installed mod. The `dir` is created if missing; existing
/// `manifest.json`, `changes.json`, `patches/`, and `files/` entries are
/// overwritten (other files in `dir` are left alone).
pub fn write_dir(dir: &Path, package: &ModPackage) -> Result<()> {
    fs::create_dir_all(dir)?;

    let manifest_path = dir.join(MANIFEST_ENTRY);
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&package.manifest)?,
    )?;

    let changes_path = dir.join(CHANGES_ENTRY);
    fs::write(
        &changes_path,
        serde_json::to_vec_pretty(package.changes.actions())?,
    )?;

    // Wipe any stale blob dirs first so removed actions don't leave orphans.
    for sub in ["patches", "files"] {
        let p = dir.join(sub);
        if p.is_dir() {
            fs::remove_dir_all(&p)?;
        }
    }

    for action in package.changes.actions() {
        if let Some((kind, bytes)) = action.op.out_of_line_bytes() {
            let blob_dir = dir.join(kind.dir_name());
            fs::create_dir_all(&blob_dir)?;
            let path = blob_dir.join(format!("{}.bin", action.id));
            fs::write(&path, bytes)?;
        }
    }

    Ok(())
}

/// Read a mod package from a directory written by [`write_dir`].
pub fn read_dir(dir: &Path) -> Result<ModPackage> {
    if !dir.is_dir() {
        return Err(ModdingError::Malformed(format!(
            "not a directory: {}",
            dir.display()
        )));
    }

    let manifest_path = dir.join(MANIFEST_ENTRY);
    if !manifest_path.is_file() {
        return Err(ModdingError::MissingEntry(MANIFEST_ENTRY.into()));
    }
    let manifest: ModManifest = serde_json::from_slice(&fs::read(&manifest_path)?)?;
    if manifest.manifest_version > MANIFEST_VERSION {
        return Err(ModdingError::UnsupportedManifestVersion(
            manifest.manifest_version,
        ));
    }

    let changes_path = dir.join(CHANGES_ENTRY);
    let mut actions: Vec<ChangeAction> = if changes_path.is_file() {
        serde_json::from_slice(&fs::read(&changes_path)?)?
    } else {
        Vec::new()
    };

    for action in actions.iter_mut() {
        let Some((kind, _)) = action.op.out_of_line_bytes() else {
            continue;
        };
        let blob_path = dir
            .join(kind.dir_name())
            .join(format!("{}.bin", action.id));
        if !blob_path.is_file() {
            return Err(ModdingError::MissingEntry(format!(
                "{}/{}.bin",
                kind.dir_name(),
                action.id
            )));
        }
        action.op.attach_blob(fs::read(&blob_path)?);
    }

    Ok(ModPackage::new(manifest, ChangeLog::from_actions(actions)))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::modding::change::ChangeOp;
    use crate::modding::value::Value;

    fn field_action() -> ChangeAction {
        ChangeAction::new(
            "CharacterInGame/MiscItem.db",
            ChangeOp::FieldDelta {
                record_id: 42,
                field: "name".into(),
                old: Value::String("Helt".into()),
                new: Value::String("Helmet".into()),
            },
        )
        .with_description("Fix typo")
    }

    fn binary_action(bytes: Vec<u8>) -> ChangeAction {
        ChangeAction::new(
            "Sprite/M_BODY1.SPR",
            ChangeOp::BinaryDelta { patch_bytes: bytes },
        )
    }

    fn replace_action(bytes: Vec<u8>) -> ChangeAction {
        ChangeAction::new("Map/cat1.map", ChangeOp::FileReplace { content: bytes })
    }

    fn add_action(bytes: Vec<u8>) -> ChangeAction {
        ChangeAction::new("Map/new_dungeon.map", ChangeOp::FileAdd { content: bytes })
    }

    fn delete_action() -> ChangeAction {
        ChangeAction::new("Sound/old.snf", ChangeOp::FileDelete)
    }

    #[test]
    fn empty_field_only_round_trips() {
        let pkg = ModPackage::new(
            ModManifest::new("typos"),
            ChangeLog::from_actions(vec![field_action()]),
        );
        let mut buf = Cursor::new(Vec::new());
        write_zip(&mut buf, &pkg).unwrap();

        buf.set_position(0);
        let back = read_zip(&mut buf).unwrap();
        assert_eq!(pkg, back);
    }

    #[test]
    fn mixed_actions_round_trip_with_blobs() {
        let pkg = ModPackage::new(
            ModManifest::new("kitchen-sink"),
            ChangeLog::from_actions(vec![
                field_action(),
                binary_action(vec![1, 2, 3, 4, 5]),
                replace_action(vec![10, 20, 30]),
                add_action(vec![100, 200]),
                delete_action(),
            ]),
        );
        let mut buf = Cursor::new(Vec::new());
        write_zip(&mut buf, &pkg).unwrap();

        buf.set_position(0);
        let back = read_zip(&mut buf).unwrap();
        assert_eq!(pkg, back);
    }

    #[test]
    fn changes_json_excludes_blob_bytes() {
        let pkg = ModPackage::new(
            ModManifest::new("bin-only"),
            ChangeLog::from_actions(vec![binary_action(vec![0xCA, 0xFE, 0xBA, 0xBE])]),
        );
        let mut buf = Cursor::new(Vec::new());
        write_zip(&mut buf, &pkg).unwrap();

        buf.set_position(0);
        let mut zip = ZipArchive::new(&mut buf).unwrap();
        let mut changes_str = String::new();
        zip.by_name(CHANGES_ENTRY)
            .unwrap()
            .read_to_string(&mut changes_str)
            .unwrap();
        assert!(
            !changes_str.contains("0xCA") && !changes_str.contains("202"),
            "bytes leaked into changes.json: {changes_str}"
        );

        // And there must be a patches/<uuid>.bin entry holding the bytes.
        let action_id = pkg.changes.actions()[0].id;
        let entry_name = format!("patches/{}.bin", action_id);
        let mut blob = Vec::new();
        zip.by_name(&entry_name)
            .unwrap()
            .read_to_end(&mut blob)
            .unwrap();
        assert_eq!(blob, vec![0xCA, 0xFE, 0xBA, 0xBE]);
    }

    #[test]
    fn missing_manifest_errors_clearly() {
        // Build a zip with only changes.json.
        let mut buf = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut buf);
            let opts = SimpleFileOptions::default();
            zip.start_file(CHANGES_ENTRY, opts).unwrap();
            zip.write_all(b"[]").unwrap();
            zip.finish().unwrap();
        }
        buf.set_position(0);
        let err = read_zip(&mut buf).unwrap_err();
        assert!(matches!(err, ModdingError::MissingEntry(ref e) if e == MANIFEST_ENTRY));
    }

    #[test]
    fn rejects_future_manifest_version() {
        let mut buf = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut buf);
            let opts = SimpleFileOptions::default();
            zip.start_file(MANIFEST_ENTRY, opts).unwrap();
            let json = format!(
                r#"{{"manifest_version":{},"name":"x"}}"#,
                MANIFEST_VERSION + 99
            );
            zip.write_all(json.as_bytes()).unwrap();
            zip.start_file(CHANGES_ENTRY, opts).unwrap();
            zip.write_all(b"[]").unwrap();
            zip.finish().unwrap();
        }
        buf.set_position(0);
        let err = read_zip(&mut buf).unwrap_err();
        assert!(matches!(err, ModdingError::UnsupportedManifestVersion(_)));
    }

    #[test]
    fn dir_round_trip() {
        let pkg = ModPackage::new(
            ModManifest::new("dir-test"),
            ChangeLog::from_actions(vec![
                field_action(),
                binary_action(vec![1, 2, 3]),
                replace_action(vec![9, 9, 9]),
                delete_action(),
            ]),
        );
        let tmp = tempfile::tempdir().unwrap();
        super::write_dir(tmp.path(), &pkg).unwrap();
        let back = super::read_dir(tmp.path()).unwrap();
        assert_eq!(pkg, back);
    }

    #[test]
    fn dir_overwrites_stale_blobs() {
        let tmp = tempfile::tempdir().unwrap();
        let p1 = ModPackage::new(
            ModManifest::new("v1"),
            ChangeLog::from_actions(vec![binary_action(vec![1, 2, 3])]),
        );
        super::write_dir(tmp.path(), &p1).unwrap();
        // Now write a different package — old blob must not linger.
        let p2 = ModPackage::new(
            ModManifest::new("v2"),
            ChangeLog::from_actions(vec![field_action()]),
        );
        super::write_dir(tmp.path(), &p2).unwrap();

        // patches/ either gone or empty.
        let patches = tmp.path().join("patches");
        if patches.exists() {
            assert!(std::fs::read_dir(&patches).unwrap().next().is_none());
        }
        let back = super::read_dir(tmp.path()).unwrap();
        assert_eq!(back.changes.len(), 1);
    }

    #[test]
    fn missing_blob_for_binary_delta_errors() {
        // Manually craft a zip declaring a BinaryDelta but omitting its blob.
        let action = binary_action(vec![]); // empty blob; we'll declare it then omit the file
        let id = action.id;

        let mut buf = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut buf);
            let opts = SimpleFileOptions::default();
            zip.start_file(MANIFEST_ENTRY, opts).unwrap();
            serde_json::to_writer(&mut zip, &ModManifest::new("x")).unwrap();
            zip.start_file(CHANGES_ENTRY, opts).unwrap();
            // Write the action (with empty patch_bytes elided by serde).
            serde_json::to_writer(&mut zip, &vec![action]).unwrap();
            // Intentionally do NOT write patches/<id>.bin.
            zip.finish().unwrap();
        }
        buf.set_position(0);
        let err = read_zip(&mut buf).unwrap_err();
        let expected = format!("patches/{}.bin", id);
        assert!(matches!(err, ModdingError::MissingEntry(ref e) if e == &expected));
    }
}
