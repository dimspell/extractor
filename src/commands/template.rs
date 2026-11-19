use std::error::Error;

use crate::commands::Command;
use crate::commands::registry::{self, FieldSpec, FieldType, FileType};

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
    let mut template = serde_json::Map::new();
    for field in &ft.fields {
        template.insert(field.name.to_string(), default_value_for(field));
    }

    serde_json::Value::Object(template)
}

fn default_value_for(field: &FieldSpec) -> serde_json::Value {
    match field.ty {
        FieldType::String => serde_json::json!(""),
        FieldType::Integer => serde_json::json!(0),
        FieldType::Number => serde_json::json!(0.0),
        FieldType::Boolean => serde_json::json!(false),
        FieldType::Array => serde_json::json!([]),
        FieldType::Object => serde_json::json!({}),
        FieldType::Null => serde_json::Value::Null,
    }
}
