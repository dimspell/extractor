use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::value::Value;

/// One file-level operation produced by the user while authoring a mod.
///
/// `FieldDelta` carries inline `old`/`new` values so it serialises whole into
/// `changes.json`. The bulky variants (`BinaryDelta`, `FileReplace`, `FileAdd`)
/// keep their bytes out-of-line: serialised `ChangeAction` values store an
/// empty/elided payload, and the package writer streams the bytes into a
/// sibling file under `patches/` or `files/` keyed by the action `id`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum ChangeOp {
    FieldDelta {
        record_id: u32,
        field: String,
        old: Value,
        new: Value,
    },
    BinaryDelta {
        /// qbsdiff patch bytes. Skipped during JSON (de)serialisation; loaded
        /// from `patches/<action_id>.bin` when reading a package.
        #[serde(skip, default)]
        patch_bytes: Vec<u8>,
    },
    FileReplace {
        #[serde(skip, default)]
        content: Vec<u8>,
    },
    FileAdd {
        #[serde(skip, default)]
        content: Vec<u8>,
    },
    FileDelete,
}

impl ChangeOp {
    /// Bytes that must be written to a sibling file when packaging.
    /// `None` for variants that fully serialise inline.
    pub fn out_of_line_bytes(&self) -> Option<(BlobKind, &[u8])> {
        match self {
            ChangeOp::BinaryDelta { patch_bytes } => Some((BlobKind::Patch, patch_bytes)),
            ChangeOp::FileReplace { content } => Some((BlobKind::File, content)),
            ChangeOp::FileAdd { content } => Some((BlobKind::File, content)),
            ChangeOp::FieldDelta { .. } | ChangeOp::FileDelete => None,
        }
    }

    /// Re-attach blob bytes loaded from disk into a freshly-deserialised op.
    pub fn attach_blob(&mut self, bytes: Vec<u8>) {
        match self {
            ChangeOp::BinaryDelta { patch_bytes } => *patch_bytes = bytes,
            ChangeOp::FileReplace { content } | ChangeOp::FileAdd { content } => *content = bytes,
            _ => {}
        }
    }

    pub fn variant_name(&self) -> &'static str {
        match self {
            ChangeOp::FieldDelta { .. } => "FieldDelta",
            ChangeOp::BinaryDelta { .. } => "BinaryDelta",
            ChangeOp::FileReplace { .. } => "FileReplace",
            ChangeOp::FileAdd { .. } => "FileAdd",
            ChangeOp::FileDelete => "FileDelete",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobKind {
    /// Goes into `patches/<id>.bin` — a qbsdiff delta.
    Patch,
    /// Goes into `files/<id>.bin` — full file content.
    File,
}

impl BlobKind {
    pub fn dir_name(self) -> &'static str {
        match self {
            BlobKind::Patch => "patches",
            BlobKind::File => "files",
        }
    }
}

/// One immutable entry in a mod's [`ChangeLog`](super::changelog::ChangeLog).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeAction {
    pub id: Uuid,
    /// Path relative to the game root, using forward slashes.
    pub file_path: String,
    pub op: ChangeOp,
    #[serde(default)]
    pub description: String,
    /// Unix epoch seconds.
    pub timestamp: i64,
}

impl ChangeAction {
    pub fn new(file_path: impl Into<String>, op: ChangeOp) -> Self {
        Self {
            id: Uuid::new_v4(),
            file_path: file_path.into(),
            op,
            description: String::new(),
            timestamp: now_secs(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
}

fn now_secs() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_delta_round_trips_inline() {
        let action = ChangeAction::new(
            "CharacterInGame/MiscItem.db",
            ChangeOp::FieldDelta {
                record_id: 42,
                field: "name".into(),
                old: Value::String("Helt".into()),
                new: Value::String("Helmet".into()),
            },
        )
        .with_description("Fix typo");

        let json = serde_json::to_string(&action).unwrap();
        let back: ChangeAction = serde_json::from_str(&json).unwrap();
        assert_eq!(action, back);
    }

    #[test]
    fn binary_delta_drops_blob_in_json() {
        let action = ChangeAction::new(
            "Sprite/M_BODY1.SPR",
            ChangeOp::BinaryDelta {
                patch_bytes: vec![1, 2, 3, 4, 5],
            },
        );
        let json = serde_json::to_string(&action).unwrap();
        assert!(
            !json.contains("patch_bytes"),
            "blob bytes must not appear in JSON: {json}"
        );

        let back: ChangeAction = serde_json::from_str(&json).unwrap();
        let ChangeOp::BinaryDelta { patch_bytes } = back.op else {
            panic!("wrong variant after round trip");
        };
        assert!(
            patch_bytes.is_empty(),
            "deserialised blob must be empty until attach_blob is called"
        );
    }

    #[test]
    fn out_of_line_bytes_routing() {
        let bin = ChangeOp::BinaryDelta {
            patch_bytes: vec![9],
        };
        assert_eq!(bin.out_of_line_bytes().unwrap().0, BlobKind::Patch);

        let rep = ChangeOp::FileReplace { content: vec![9] };
        assert_eq!(rep.out_of_line_bytes().unwrap().0, BlobKind::File);

        let add = ChangeOp::FileAdd { content: vec![9] };
        assert_eq!(add.out_of_line_bytes().unwrap().0, BlobKind::File);

        let del = ChangeOp::FileDelete;
        assert!(del.out_of_line_bytes().is_none());

        let fd = ChangeOp::FieldDelta {
            record_id: 0,
            field: "x".into(),
            old: Value::Null,
            new: Value::Null,
        };
        assert!(fd.out_of_line_bytes().is_none());
    }

    #[test]
    fn attach_blob_repopulates() {
        let mut op = ChangeOp::FileReplace { content: vec![] };
        op.attach_blob(vec![1, 2, 3]);
        let ChangeOp::FileReplace { content } = op else {
            unreachable!()
        };
        assert_eq!(content, vec![1, 2, 3]);
    }
}
