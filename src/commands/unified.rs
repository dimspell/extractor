use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use crate::commands::registry::{self, DetectResult, FileType};
use crate::commands::Command;

#[derive(clap::Args, Clone)]
pub struct ExtractArgs {
    /// Path to game file
    #[arg(short, long)]
    pub input: String,

    /// Output file path (default: stdout)
    #[arg(short, long)]
    pub output: Option<String>,

    /// File type override
    #[arg(short, long)]
    pub r#type: Option<String>,

    /// Pretty-print JSON
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(clap::Args, Clone)]
pub struct PatchArgs {
    /// Path to JSON file
    #[arg(short, long)]
    pub input: String,

    /// Path to game file to patch
    #[arg(short, long)]
    pub target: String,

    /// Output path (if different from target)
    #[arg(short, long)]
    pub output: Option<String>,

    /// File type override
    #[arg(long)]
    pub r#type: Option<String>,

    /// Validate without writing
    #[arg(short, long)]
    pub dry_run: bool,

    /// Patch target directly, create .bak backup
    #[arg(long)]
    pub in_place: bool,

    /// Skip backup creation (with --in-place)
    #[arg(long)]
    pub no_backup: bool,
}

pub struct ExtractCommand {
    pub args: ExtractArgs,
}

pub struct PatchCommand {
    pub args: PatchArgs,
}

impl Command for ExtractCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        let input_path = Path::new(&self.args.input);

        if !input_path.exists() {
            return Err(format!("File not found: {}", input_path.display()).into());
        }

        let file_type = resolve_type(&self.args.r#type, input_path, None)?;

        let data = (file_type.extract_fn)(input_path)
            .map_err(|e| format!("Failed to extract {}: {}", input_path.display(), e))?;

        let record_count = count_records(&data);
        let fields = extract_fields(&data);

        let output = serde_json::json!({
            "_meta": {
                "file_type": file_type.key,
                "record_count": record_count,
                "fields": fields,
            },
            "data": data,
        });

        let json_str = if self.args.pretty {
            serde_json::to_string_pretty(&output)
                .map_err(|e| format!("Failed to serialize JSON: {}", e))?
        } else {
            serde_json::to_string(&output)
                .map_err(|e| format!("Failed to serialize JSON: {}", e))?
        };

        if let Some(output_path) = &self.args.output {
            fs::write(output_path, &json_str)
                .map_err(|e| format!("Failed to write to {}: {}", output_path, e))?;
            eprintln!("Extracted {} records to {}", record_count, output_path);
        } else {
            println!("{}", json_str);
        }

        Ok(())
    }
}

impl Command for PatchCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        let input_path = Path::new(&self.args.input);
        let target_path = Path::new(&self.args.target);

        if !input_path.exists() {
            return Err(format!("JSON file not found: {}", input_path.display()).into());
        }

        let json_data = fs::read_to_string(input_path)
            .map_err(|e| format!("Failed to read {}: {}", input_path.display(), e))?;

        let data: serde_json::Value = serde_json::from_str(&json_data)
            .map_err(|e| format!("Invalid JSON in {}: {}", input_path.display(), e))?;

        // Unwrap { "_meta": ..., "data": [...] } format if present
        let patch_data = if let Some(inner) = data.get("data") {
            inner.clone()
        } else {
            data.clone()
        };

        let meta_hint = data
            .get("_meta")
            .and_then(|m| m.get("file_type"))
            .and_then(|v| v.as_str());
        let file_type = resolve_type(&self.args.r#type, target_path, meta_hint)?;

        if self.args.dry_run {
            if let Some(validate_fn) = file_type.validate_fn {
                match validate_fn(&patch_data) {
                    Ok(()) => {
                        eprintln!(
                            "Validation passed: {} is valid for type '{}'",
                            input_path.display(),
                            file_type.key
                        );
                        return Ok(());
                    }
                    Err(errors) => {
                        let error_json = serde_json::json!({
                            "valid": false,
                            "error_count": errors.len(),
                            "errors": errors,
                        });
                        return Err(format!(
                            "Validation failed ({} error(s)):\n{}",
                            errors.len(),
                            serde_json::to_string_pretty(&error_json).unwrap()
                        )
                        .into());
                    }
                }
            } else {
                eprintln!(
                    "No validation available for type '{}', checking JSON parse only",
                    file_type.key
                );
                return Ok(());
            }
        }

        let output_path = if let Some(ref output) = self.args.output {
            PathBuf::from(output)
        } else if self.args.in_place {
            if !self.args.no_backup {
                let backup_path = target_path.with_extension(
                    target_path
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| format!("{}.bak", e))
                        .as_deref()
                        .unwrap_or("bak"),
                );
                fs::copy(target_path, &backup_path).map_err(|e| {
                    format!(
                        "Failed to create backup at {}: {}. Use --no-backup to skip, or --output to write elsewhere.",
                        backup_path.display(),
                        e
                    )
                })?;
                eprintln!("Backup created: {}", backup_path.display());
            }
            target_path.to_path_buf()
        } else {
            target_path.to_path_buf()
        };

        (file_type.patch_fn)(&patch_data, &output_path)
            .map_err(|e| format!("Failed to patch {}: {}", output_path.display(), e))?;

        eprintln!(
            "Patched {} with {} records",
            output_path.display(),
            count_records(&patch_data)
        );
        Ok(())
    }
}

fn resolve_type(
    type_override: &Option<String>,
    path: &Path,
    meta_hint: Option<&str>,
) -> Result<&'static FileType, Box<dyn Error>> {
    if let Some(key) = type_override {
        registry::get_by_key(key)
            .map(|ft| ft as &'static FileType)
            .ok_or_else(|| {
                let suggestions = registry::suggest_similar_keys(key);
                if suggestions.is_empty() {
                    format!("Unknown file type '{key}'. Use 'list' to see all supported types.")
                } else {
                    format!(
                        "Unknown file type '{key}'. Did you mean: {}?",
                        suggestions.join(", ")
                    )
                }
                .into()
            })
    } else {
        match registry::detect(path) {
            DetectResult::Single(ft) => Ok(ft),
            DetectResult::Ambiguous(candidates) => {
                // Use the file_type embedded in the extracted JSON's _meta block if available
                if let Some(hint) = meta_hint {
                    if let Some(ft) = registry::get_by_key(hint) {
                        return Ok(ft);
                    }
                }
                let keys: Vec<&str> = candidates.iter().map(|ft| ft.key).collect();
                Err(format!(
                    "Cannot auto-detect type for '{}': filename doesn't match any known name.\n\
                     Possible types: {}\n\
                     Use --type <key> to specify (e.g. --type {})",
                    path.display(),
                    keys.join(", "),
                    keys.first().copied().unwrap_or("?")
                )
                .into())
            }
            DetectResult::None => {
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("(none)");
                Err(format!(
                    "No supported file type for extension '.{ext}' in '{}'.\n\
                     Use 'list' to see all supported types.",
                    path.display()
                )
                .into())
            }
        }
    }
}

fn count_records(data: &serde_json::Value) -> usize {
    match data {
        serde_json::Value::Array(arr) => arr.len(),
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::Array(arr)) = map.get("data") {
                arr.len()
            } else if let Some(serde_json::Value::Array(arr)) = map.get("records") {
                arr.len()
            } else {
                1
            }
        }
        _ => 1,
    }
}

/// Extract field names from the first record in the data.
fn extract_fields(data: &serde_json::Value) -> Vec<String> {
    let first_record = match data {
        serde_json::Value::Array(arr) => arr.first(),
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::Array(arr)) = map.get("data") {
                arr.first()
            } else if let Some(serde_json::Value::Array(arr)) = map.get("records") {
                arr.first()
            } else {
                None
            }
        }
        _ => None,
    };

    match first_record {
        Some(serde_json::Value::Object(obj)) => obj.keys().cloned().collect(),
        _ => vec![],
    }
}
