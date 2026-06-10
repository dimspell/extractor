use std::error::Error;

use crate::commands::registry::{self, FileType};
use crate::commands::Command;

#[derive(clap::Args, Clone)]
pub struct TemplateArgs {
    /// File type
    #[arg(long)]
    pub r#type: String,

    /// Pretty-print JSON
    #[arg(short, long)]
    pub pretty: bool,
}

pub struct TemplateCommand {
    pub args: TemplateArgs,
}

impl Command for TemplateCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        let file_type = registry::get_by_key(&self.args.r#type).ok_or_else(|| {
            format!(
                "Unknown file type '{}'. Available types:\n{}",
                self.args.r#type,
                registry::format_type_list()
            )
        })?;

        let template = generate_template(file_type);
        let output = if self.args.pretty {
            serde_json::to_string_pretty(&template)
                .map_err(|e| format!("Failed to serialize JSON: {}", e))?
        } else {
            serde_json::to_string(&template)
                .map_err(|e| format!("Failed to serialize JSON: {}", e))?
        };

        println!("{}", output);
        Ok(())
    }
}

fn generate_template(ft: &FileType) -> serde_json::Value {
    let fields = registry::get_type_fields(ft.key);

    let mut template = serde_json::Map::new();
    for field in &fields {
        template.insert(field.clone(), default_value_for_field(field));
    }

    serde_json::Value::Object(template)
}

fn default_value_for_field(field: &str) -> serde_json::Value {
    if field == "id" {
        return serde_json::json!(0);
    }
    if field.contains("name") || field.contains("description") || field.contains("filename") {
        return serde_json::json!("");
    }
    if field.contains("flag") || field.starts_with("is_") || field.starts_with("has_") {
        return serde_json::json!(false);
    }
    serde_json::json!(0)
}


