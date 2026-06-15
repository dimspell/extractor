use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use dispel_core::references::event_scr::EventScript;
use dispel_core::references::extractor::Extractor;

use crate::commands::Command;

#[derive(clap::Args, Clone)]
pub struct ModPackArgs {
    /// Path to the Dispel game directory (contains Ref/Event*.scr)
    #[arg(short, long)]
    pub game_path: String,

    /// Output directory for JSON files
    #[arg(short, long, default_value = "mod-pack")]
    pub output: String,

    /// Pretty-print JSON
    #[arg(short, long)]
    pub pretty: bool,

    /// Output as a single JSON array file instead of individual files
    #[arg(short, long)]
    pub single_file: bool,
}

pub struct ModPackCommand {
    pub args: ModPackArgs,
}

impl Command for ModPackCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        let game_dir = Path::new(&self.args.game_path);
        let ref_dir = game_dir.join("Ref");

        if !ref_dir.exists() {
            return Err(format!("Ref directory not found at: {}", ref_dir.display()).into());
        }

        // Find all .scr files in the Ref directory (case-insensitive)
        let mut scr_files: Vec<PathBuf> = Vec::new();
        for entry in fs::read_dir(&ref_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("scr") {
                scr_files.push(path);
            }
        }

        scr_files.sort();

        if scr_files.is_empty() {
            return Err("No .scr files found in Ref directory".into());
        }

        eprintln!(
            "Found {} event script files in {}",
            scr_files.len(),
            ref_dir.display()
        );

        let output_dir = Path::new(&self.args.output);

        if self.args.single_file {
            self.export_single_file(&scr_files, output_dir)?;
        } else {
            self.export_individual_files(&scr_files, output_dir)?;
        }

        Ok(())
    }
}

impl ModPackCommand {
    fn export_single_file(
        &self,
        scr_files: &[PathBuf],
        output_dir: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let mut all_scripts: Vec<serde_json::Value> = Vec::with_capacity(scr_files.len());

        for path in scr_files {
            match EventScript::read_file(path) {
                Ok(records) => {
                    for record in records {
                        let value = serde_json::to_value(&record)?;
                        all_scripts.push(value);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                }
            }
        }

        let output = serde_json::json!({
            "_meta": {
                "format": "dispel-event-scripts",
                "version": 1,
                "record_count": all_scripts.len(),
                "description": "Batch export of all Event*.scr files",
            },
            "data": all_scripts,
        });

        fs::create_dir_all(output_dir)?;
        let output_path = output_dir.join("event_scripts.json");

        let json_str = if self.args.pretty {
            serde_json::to_string_pretty(&output)?
        } else {
            serde_json::to_string(&output)?
        };

        fs::write(&output_path, &json_str)?;
        eprintln!(
            "Exported {} event scripts to {}",
            all_scripts.len(),
            output_path.display()
        );

        Ok(())
    }

    fn export_individual_files(
        &self,
        scr_files: &[PathBuf],
        output_dir: &Path,
    ) -> Result<(), Box<dyn Error>> {
        fs::create_dir_all(output_dir)?;

        let mut success_count = 0;
        let mut fail_count = 0;

        for path in scr_files {
            match EventScript::read_file(path) {
                Ok(records) => {
                    for record in records {
                        let json_str = if self.args.pretty {
                            serde_json::to_string_pretty(&record)?
                        } else {
                            serde_json::to_string(&record)?
                        };

                        let filename = format!("Event{:04}.json", record.id);
                        let output_path = output_dir.join(&filename);
                        fs::write(&output_path, &json_str)?;
                        success_count += 1;
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                    fail_count += 1;
                }
            }
        }

        eprintln!(
            "Exported {success_count} event scripts to {} ({} failed)",
            output_dir.display(),
            fail_count,
        );

        Ok(())
    }
}
