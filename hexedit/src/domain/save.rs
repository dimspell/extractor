//! Domain-level types for the save boundary between the editor and its host.
//!
//! These types deliberately contain no `iced` message or widget types: they
//! are plain data describing *what* was saved and *how* it went. The host
//! facing callback (`SavePort`, defined in `config.rs`) carries these across
//! the boundary, so embedding applications never have to construct or match
//! on hexedit's internal `HexEditorMessage`.

use std::path::PathBuf;

/// A snapshot of the open document handed to the host for persistence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveRequest {
    /// Path of the file being edited (used by hosts to resolve mod-relative paths).
    pub path: PathBuf,
    /// Full current contents of the edited buffer.
    pub bytes: Vec<u8>,
}

/// The result of a save attempt, returned by the host to the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveOutcome {
    /// The save succeeded. Carries a human-readable status message.
    Saved(String),
    /// The save failed. Carries the error description.
    Failed(String),
}

impl SaveOutcome {
    /// Convert a host-side `Result` into the domain-level save outcome.
    pub fn from_result(result: Result<String, String>) -> Self {
        match result {
            Ok(message) => Self::Saved(message),
            Err(error) => Self::Failed(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_outcome_from_result_maps_both_variants() {
        assert_eq!(
            SaveOutcome::from_result(Ok("done".into())),
            SaveOutcome::Saved("done".into())
        );
        assert_eq!(
            SaveOutcome::from_result(Err("disk full".into())),
            SaveOutcome::Failed("disk full".into())
        );
    }
}
