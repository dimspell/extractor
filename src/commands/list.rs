use std::error::Error;

use serde::Serialize;

use crate::cli::ListArgs;
use crate::commands::Command;
use crate::commands::registry;
use crate::commands::registry::FileType;

impl Command for ListArgs {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        let types: Vec<&FileType> = registry::FILE_TYPES
            .iter()
            .filter(|ft| {
                if let Some(ref filter) = self.filter {
                    let filter_lower = filter.to_lowercase();
                    ft.key.to_lowercase().contains(&filter_lower)
                        || ft.name.to_lowercase().contains(&filter_lower)
                        || ft.description.to_lowercase().contains(&filter_lower)
                } else {
                    true
                }
            })
            .collect();

        match self.format.as_str() {
            "json" => {
                let output = ListOutput {
                    types: types.iter().map(|ft| TypeInfo::from(*ft)).collect(),
                };
                println!(
                    "{}",
                    serde_json::to_string_pretty(&output)
                        .map_err(|e| format!("Failed to serialize JSON: {}", e))?
                );
            }
            _ => {
                if types.is_empty() {
                    println!("No matching file types found.");
                    return Ok(());
                }

                println!("{:<20} {:<18} {:<8} Description", "Type", "Name", "Ext");
                println!("{}", "-".repeat(80));
                for ft in &types {
                    let ext = ft.extensions.join(", ");
                    println!(
                        "{:<20} {:<18} {:<8} {}",
                        ft.key, ft.name, ext, ft.description
                    );
                }
                println!("\n{} file types listed", types.len());
            }
        }

        Ok(())
    }
}

#[derive(Serialize)]
struct ListOutput {
    types: Vec<TypeInfo>,
}

#[derive(Serialize)]
struct TypeInfo {
    name: &'static str,
    description: &'static str,
    extensions: Vec<&'static str>,
    record_type: &'static str,
    editor: &'static str,
    fields: Vec<String>,
}

impl From<&FileType> for TypeInfo {
    fn from(ft: &FileType) -> Self {
        TypeInfo {
            name: ft.key,
            description: ft.description,
            extensions: ft.extensions.to_vec(),
            record_type: ft.record_type,
            editor: ft.editor,
            fields: ft.field_names().map(str::to_string).collect(),
        }
    }
}
