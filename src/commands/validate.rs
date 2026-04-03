use std::error::Error;
use std::fs;
use std::path::Path;

use crate::commands::registry;
use crate::commands::Command;

#[derive(clap::Args, Clone)]
pub struct ValidateArgs {
    /// Path to JSON file
    #[arg(short, long)]
    pub input: String,

    /// File type
    #[arg(long)]
    pub r#type: String,

    /// Verbose output
    #[arg(long)]
    pub verbose: bool,
}

pub struct ValidateCommand {
    pub args: ValidateArgs,
}

impl Command for ValidateCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        let input_path = Path::new(&self.args.input);

        if !input_path.exists() {
            return Err(format!("File not found: {}", input_path.display()).into());
        }

        let file_type = registry::get_by_key(&self.args.r#type).ok_or_else(|| {
            format!(
                "Unknown file type '{}'. Available types:\n{}",
                self.args.r#type,
                registry::format_type_list()
            )
        })?;

        let json_data = fs::read_to_string(input_path)
            .map_err(|e| format!("Failed to read {}: {}", input_path.display(), e))?;

        let data: serde_json::Value = serde_json::from_str(&json_data)
            .map_err(|e| format!("Invalid JSON in {}: {}", input_path.display(), e))?;

        // Unwrap { "_meta": ..., "data": [...] } format if present
        let validate_data = if let Some(inner) = data.get("data") {
            inner.clone()
        } else {
            data.clone()
        };

        if let Some(validate_fn) = file_type.validate_fn {
            match validate_fn(&validate_data) {
                Ok(()) => {
                    println!(
                        "Valid: {} matches '{}' format",
                        input_path.display(),
                        file_type.key
                    );
                    Ok(())
                }
                Err(errors) => {
                    if self.args.verbose {
                        let error_json = serde_json::json!({
                            "valid": false,
                            "type": file_type.key,
                            "errors": errors
                        });
                        eprintln!("{}", serde_json::to_string_pretty(&error_json).unwrap());
                    } else {
                        for err in &errors {
                            eprintln!("Error: {}", err);
                        }
                    }
                    Err(format!("Validation failed: {} errors found", errors.len()).into())
                }
            }
        } else {
            eprintln!(
                "No validation available for type '{}'. JSON parse succeeded.",
                file_type.key
            );
            Ok(())
        }
    }

    fn name(&self) -> &'static str {
        "validate"
    }

    fn description(&self) -> &'static str {
        "Validate JSON against file format schema"
    }
}
