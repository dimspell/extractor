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

    fn name(&self) -> &'static str {
        "template"
    }

    fn description(&self) -> &'static str {
        "Generate a minimal JSON template for a file type"
    }
}

fn generate_template(ft: &FileType) -> serde_json::Value {
    let fields = get_type_fields(ft);

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

fn get_type_fields(ft: &FileType) -> Vec<String> {
    match ft.key {
        "weapons" => vec![
            "id",
            "name",
            "description",
            "base_price",
            "health_points",
            "mana_points",
            "strength",
            "agility",
            "wisdom",
            "constitution",
            "to_dodge",
            "to_hit",
            "attack",
            "defense",
            "magical_strength",
            "durability",
            "req_strength",
            "req_agility",
            "req_wisdom",
        ],
        "monsters" => vec![
            "id",
            "name",
            "ai_type",
            "health_points_min",
            "health_points_max",
            "mana_points_min",
            "mana_points_max",
            "offense_min",
            "offense_max",
            "defense_min",
            "defense_max",
            "to_hit_min",
            "to_hit_max",
            "to_dodge_min",
            "to_dodge_max",
            "magic_attack_min",
            "magic_attack_max",
            "attack_speed",
            "walk_speed",
            "boldness",
            "magic_level",
            "known_spell_slot1",
            "known_spell_slot2",
            "known_spell_slot3",
            "special_attack",
            "special_attack_chance",
            "special_attack_duration",
            "exp_gain_min",
            "exp_gain_max",
            "gold_drop_min",
            "gold_drop_max",
            "detection_sight_size",
            "distance_range_size",
            "is_oversize",
            "is_undead",
            "has_blood",
        ],
        "all_maps" => vec![
            "id",
            "map_filename",
            "map_name",
            "pgp_filename",
            "dlg_filename",
            "lighting",
        ],
        "map_ini" => vec![
            "id",
            "start_x",
            "start_y",
            "start_dir",
            "monsters",
            "npcs",
            "extras",
            "camera_event",
            "cd_track",
        ],
        "monster_ini" => vec![
            "id",
            "name",
            "sprite_filename",
            "attack",
            "hit",
            "death",
            "walking",
            "casting_magic",
        ],
        "npc_ini" => vec![
            "id",
            "name",
            "sprite_filename",
            "attack",
            "hit",
            "death",
            "walking",
            "casting_magic",
        ],
        "quest" => vec!["id", "type_id", "title", "description"],
        "message" => vec!["id", "text"],
        "magic" => vec![
            "id",
            "name",
            "description",
            "mp_cost",
            "spell_level",
            "school",
            "target_type",
            "flag",
            "constant",
            "effect",
        ],
        "store" => vec![
            "id",
            "name",
            "product_type",
            "product_id",
            "price",
            "haggle_min",
            "haggle_max",
            "dialog_id",
        ],
        _ => vec!["id"],
    }
    .into_iter()
    .map(String::from)
    .collect()
}
