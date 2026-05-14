use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModdingError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("malformed mod package: {0}")]
    Malformed(String),

    #[error("missing required entry in package: {0}")]
    MissingEntry(String),

    #[error("unsupported manifest version: {0}")]
    UnsupportedManifestVersion(u32),
}

pub type Result<T> = std::result::Result<T, ModdingError>;
