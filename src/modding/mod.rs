//! Mod authoring, packaging, and apply pipeline.
//!
//! Pure data + I/O. No GUI dependencies.
//!
//! Phase 0 introduces the data model ([`change`], [`changelog`], [`manifest`],
//! [`value`]). Phase 1 layers [`package`] read/write of the on-disk `mod.zip`
//! format on top.

pub mod change;
pub mod changelog;
pub mod error;
pub mod manifest;
pub mod package;
pub mod value;

pub use change::{BlobKind, ChangeAction, ChangeOp};
pub use changelog::{ChangeLog, HISTORY_CAP};
pub use error::{ModdingError, Result};
pub use manifest::{ModManifest, MANIFEST_VERSION};
pub use package::{read_zip, write_zip, ModPackage};
pub use value::Value;
