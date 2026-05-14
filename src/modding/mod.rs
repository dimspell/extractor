//! Mod authoring, packaging, and apply pipeline.
//!
//! Pure data + I/O. No GUI dependencies.
//!
//! Phase 0 introduces the data model ([`change`], [`changelog`], [`manifest`],
//! [`value`]). Phase 1 layers [`package`] read/write of the on-disk `mod.zip`
//! format on top.

pub mod apply;
pub mod bsdiff;
pub mod change;
pub mod changelog;
pub mod conflicts;
pub mod error;
pub mod manifest;
pub mod package;
pub mod patcher;
pub mod patchers;
pub mod registry;
pub mod resolution;
pub mod value;
pub mod vanilla;
pub mod workspace;

pub use apply::{apply_all, revert_to_vanilla, ApplyReport, ModEntry, RevertReport};
pub use bsdiff::{apply_delta, make_delta};
pub use change::{BlobKind, ChangeAction, ChangeOp};
pub use changelog::{ChangeLog, HISTORY_CAP};
pub use conflicts::{detect_conflicts, Conflict, ConflictKind, ConflictParticipant};
pub use error::{ModdingError, Result};
pub use manifest::{ModManifest, MANIFEST_VERSION};
pub use package::{read_zip, write_zip, ModPackage};
pub use patcher::RecordPatcher;
pub use registry::PatcherRegistry;
pub use resolution::{FieldKey, ResolutionMap};
pub use value::Value;
pub use vanilla::VanillaStore;
pub use workspace::{InstalledMod, Workspace};
