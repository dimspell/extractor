//! Mod authoring, packaging, and apply pipeline.
//!
//! Pure data + I/O. No GUI dependencies.
//!
//! The data model ([`change`], [`changelog`], [`manifest`], [`value`]) and
//! [`package`] read/write of the on-disk `mod.zip` format live here.

pub mod apply;
pub mod bsdiff;
pub mod change;
pub mod changelog;
pub mod conflicts;
pub mod error;
pub mod facade;
pub mod fs;
pub mod manifest;
pub mod package;
pub mod patcher;
pub mod patchers;
pub mod registry;
pub mod resolution;
#[cfg(test)]
mod test_helpers;
pub mod value;
pub mod vanilla;
pub mod workspace;

pub use apply::{ApplyReport, ModEntry, RevertReport, apply_all, revert_to_vanilla};
pub use bsdiff::{apply_delta, make_delta};
pub use change::{BlobKind, ChangeAction, ChangeOp, FieldChange};
pub use changelog::{ChangeLog, ChangeTarget, HISTORY_CAP, effective_actions};
pub use conflicts::{Conflict, ConflictKind, ConflictParticipant, detect_conflicts};
pub use error::{ModdingError, Result};
pub use facade::{Modding, ReconcileReport, decide_file_op};
pub use fs::{Fs, FsHandle, FsMetadata, MemFs, OsFs};
pub use manifest::{MANIFEST_VERSION, ModManifest};
pub use package::{ModPackage, read_zip, write_zip};
pub use patcher::RecordPatcher;
pub use registry::PatcherRegistry;
pub use resolution::{FieldKey, ResolutionMap};
pub use value::Value;
pub use vanilla::VanillaStore;
pub use workspace::{InstalledMod, Workspace};
