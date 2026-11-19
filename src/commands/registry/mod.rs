pub mod detect;
pub(crate) mod entries;
pub mod types;

// Re-export public API
pub use detect::{FILE_TYPES, detect, format_type_list, get_by_key, suggest_similar_keys};
pub use types::{DetectResult, FieldSpec, FieldType, FileType};
