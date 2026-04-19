pub mod detect;
pub(crate) mod entries;
pub mod types;

// Re-export public API
pub use detect::{detect, format_type_list, get_by_key, suggest_similar_keys, FILE_TYPES};
pub use types::{DetectResult, FileType};
