use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::references::extractor::Extractor;

/// Detection strategy for a file type.
pub(crate) enum DetectKind {
    /// Match by known INI filename (case-insensitive).
    Ini(&'static str),
    /// Match by known DB filename (case-insensitive).
    Db(&'static [&'static str]),
    /// Match by known REF filename prefix (case-insensitive).
    RefPrefix(&'static str),
    /// Match by known SCR filename (case-insensitive).
    Scr(&'static str),
    /// Match by known DLG filename prefix (case-insensitive).
    DlgPrefix(&'static str),
    /// Match by known PGP filename prefix (case-insensitive).
    PgpPrefix(&'static str),
}

/// Function pointer types for the file type registry.
pub(crate) type ExtractFn = fn(&Path) -> Result<serde_json::Value, Box<dyn std::error::Error>>;
pub(crate) type PatchFn = fn(&serde_json::Value, &Path) -> Result<(), Box<dyn std::error::Error>>;
pub(crate) type ValidateFn = fn(&serde_json::Value) -> Result<(), Vec<String>>;

/// A registered file type with detection, extraction, and patching capabilities.
pub struct FileType {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub extensions: &'static [&'static str],
    pub(crate) detect_kind: DetectKind,
    pub extract_fn: ExtractFn,
    pub patch_fn: PatchFn,
    pub validate_fn: Option<ValidateFn>,
}

impl FileType {
    pub fn matches(&self, path: &Path) -> bool {
        match self.detect_kind {
            DetectKind::Ini(name) => detect_filename(path, name),
            DetectKind::Db(names) => detect_db_filename(path, names),
            DetectKind::RefPrefix(prefix) => detect_filename_prefix(path, prefix),
            DetectKind::Scr(name) => detect_filename(path, name),
            DetectKind::DlgPrefix(prefix) => detect_filename_prefix(path, prefix),
            DetectKind::PgpPrefix(prefix) => detect_filename_prefix(path, prefix),
        }
    }
}

/// Result of file type detection.
pub enum DetectResult {
    /// Exactly one type matched.
    Single(&'static FileType),
    /// No type matched.
    None,
}

impl DetectResult {
    pub fn into_single(self) -> Option<&'static FileType> {
        match self {
            DetectResult::Single(ft) => Some(ft),
            DetectResult::None => None,
        }
    }
}

// ===========================================================================
// Detection helpers
// ===========================================================================

/// Check if the filename matches one of the known names (case-insensitive).
pub(crate) fn detect_db_filename(path: &Path, names: &[&str]) -> bool {
    let file_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(n) => n.to_lowercase(),
        None => return false,
    };
    names.iter().any(|&name| file_name == name.to_lowercase())
}

/// Check if the filename starts with the given prefix (case-insensitive).
pub(crate) fn detect_filename_prefix(path: &Path, prefix: &str) -> bool {
    let file_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(n) => n.to_lowercase(),
        None => return false,
    };
    file_name.starts_with(&prefix.to_lowercase())
}

/// Check if the filename matches exactly (case-insensitive).
pub(crate) fn detect_filename(path: &Path, name: &str) -> bool {
    let file_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(n) => n.to_lowercase(),
        None => return false,
    };
    file_name == name.to_lowercase()
}

// ===========================================================================
// Generic extract/patch/validate helpers
// ===========================================================================

pub(crate) fn extract_as<T>(path: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error>>
where
    T: Extractor + Serialize,
{
    let records = T::read_file(path)?;
    let value = serde_json::to_value(&records)?;
    Ok(value)
}

pub(crate) fn patch_as<T>(
    data: &serde_json::Value,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: Extractor + for<'de> Deserialize<'de>,
{
    let records: Vec<T> = serde_json::from_value(data.clone())?;
    T::save_file(&records, path)?;
    Ok(())
}

pub(crate) fn validate_as<T>(data: &serde_json::Value) -> Result<(), Vec<String>>
where
    T: for<'de> Deserialize<'de>,
{
    let result: Result<Vec<T>, serde_json::Error> = serde_json::from_value(data.clone());
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(vec![format!("Deserialization error: {}", e)]),
    }
}

pub(crate) fn patch_not_supported(
    _data: &serde_json::Value,
    _path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    Err("Patch operation not supported for this file type".into())
}

pub(crate) fn extract_map_file(
    path: &Path,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    use crate::map;
    use std::fs::File;
    use std::io::BufReader;

    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let map_data = map::read_map_data(&mut reader)?;
    let json = map_data.to_json();
    Ok(serde_json::to_value(&json)?)
}

pub(crate) fn extract_tileset(
    path: &Path,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    use crate::map::tileset;

    let tiles = tileset::extract(path)?;
    let tile_entries: Vec<serde_json::Value> = tiles
        .iter()
        .enumerate()
        .map(|(i, _tile)| {
            serde_json::json!({
                "index": i,
                "pixels": null,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "tile_count": tiles.len(),
        "tile_width": 32,
        "tile_height": 32,
        "rendered_width": 62,
        "rendered_height": 32,
        "color_format": "RGB565",
        "tiles": tile_entries,
        "note": "Pixel data omitted. Use 'map tiles' command to extract individual tile images.",
    }))
}

pub(crate) fn extract_sprite_info(
    path: &Path,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    use crate::sprite;
    let info = sprite::get_sprite_info(path)?;
    Ok(serde_json::to_value(&info)?)
}
