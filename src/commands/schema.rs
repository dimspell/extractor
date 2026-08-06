use std::error::Error;

use crate::commands::Command;
use crate::commands::registry::{self, FileType};

#[derive(clap::Args, Clone)]
pub struct SchemaArgs {
    /// File type
    #[arg(long)]
    pub r#type: String,
}

pub struct SchemaCommand {
    pub args: SchemaArgs,
}

impl Command for SchemaCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        let file_type = registry::get_by_key(&self.args.r#type).ok_or_else(|| {
            format!(
                "Unknown file type '{}'. Available types:\n{}",
                self.args.r#type,
                registry::format_type_list()
            )
        })?;

        let schema = generate_json_schema(file_type);
        println!(
            "{}",
            serde_json::to_string_pretty(&schema)
                .map_err(|e| format!("Failed to serialize JSON schema: {}", e))?
        );

        Ok(())
    }
}

// ===========================================================================
// Schema generation
// ===========================================================================

fn generate_json_schema(ft: &FileType) -> serde_json::Value {
    match ft.key {
        "map_file" => return generate_map_schema(),
        "gtl" | "btl" => return generate_tileset_schema(ft.key),
        _ => {}
    }

    let fields = registry::get_type_fields(ft.key);

    let mut properties = serde_json::Map::new();
    for field in &fields {
        properties.insert(field.clone(), infer_json_type(field));
    }

    serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": format!("{} records", ft.key),
        "description": ft.description,
        "type": "array",
        "items": {
            "type": "object",
            "properties": properties,
            "required": fields,
        }
    })
}

fn generate_map_schema() -> serde_json::Value {
    serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "DispelMapData",
        "description": "Complete parsed representation of a Dispel game .MAP file",
        "type": "object",
        "required": ["metadata", "gtl_tiles", "btl_tiles", "collisions", "events", "tiled_objects", "sprites", "internal_sprites"],
        "properties": {
            "metadata": {
                "type": "object",
                "required": ["tiled_width", "tiled_height"],
                "properties": {
                    "chunk_width": { "type": "integer", "description": "Number of 25-tile chunks on X axis" },
                    "chunk_height": { "type": "integer", "description": "Number of 25-tile chunks on Y axis" },
                    "tiled_width": { "type": "integer", "description": "Total tile count on X axis" },
                    "tiled_height": { "type": "integer", "description": "Total tile count on Y axis" },
                    "map_width_in_pixels": { "type": "integer" },
                    "map_height_in_pixels": { "type": "integer" },
                    "non_occluded_start_x": { "type": "integer" },
                    "non_occluded_start_y": { "type": "integer" },
                    "occluded_width": { "type": "integer" },
                    "occluded_height": { "type": "integer" }
                }
            },
            "gtl_tiles": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["x", "y", "tile_id"],
                    "properties": {
                        "x": { "type": "integer", "minimum": 0 },
                        "y": { "type": "integer", "minimum": 0 },
                        "tile_id": { "type": "integer", "description": "Index into the .GTL tileset" }
                    }
                }
            },
            "btl_tiles": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["x", "y", "tile_id"],
                    "properties": {
                        "x": { "type": "integer", "minimum": 0 },
                        "y": { "type": "integer", "minimum": 0 },
                        "tile_id": { "type": "integer", "description": "Index into the .BTL tileset" }
                    }
                }
            },
            "collisions": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["x", "y", "blocked"],
                    "properties": {
                        "x": { "type": "integer", "minimum": 0 },
                        "y": { "type": "integer", "minimum": 0 },
                        "blocked": { "type": "boolean" }
                    }
                }
            },
            "events": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["x", "y", "event_id"],
                    "properties": {
                        "x": { "type": "integer", "minimum": 0 },
                        "y": { "type": "integer", "minimum": 0 },
                        "event_id": { "type": "integer" }
                    }
                }
            },
            "tiled_objects": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["index", "x", "y", "tile_ids"],
                    "properties": {
                        "index": { "type": "integer" },
                        "x": { "type": "integer" },
                        "y": { "type": "integer" },
                        "tile_ids": { "type": "array", "items": { "type": "integer" } }
                    }
                }
            },
            "sprites": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["index", "sprite_id", "x", "y"],
                    "properties": {
                        "index": { "type": "integer" },
                        "sprite_id": { "type": "integer" },
                        "x": { "type": "integer" },
                        "y": { "type": "integer" }
                    }
                }
            },
            "internal_sprites": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["index", "image_stamp", "frame_count", "frames"],
                    "properties": {
                        "index": { "type": "integer" },
                        "image_stamp": { "type": "integer", "enum": [6, 9] },
                        "frame_count": { "type": "integer" },
                        "frames": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["width", "height", "origin_x", "origin_y"],
                                "properties": {
                                    "width": { "type": "integer", "minimum": 0 },
                                    "height": { "type": "integer", "minimum": 0 },
                                    "origin_x": { "type": "integer" },
                                    "origin_y": { "type": "integer" }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

fn generate_tileset_schema(key: &str) -> serde_json::Value {
    serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "DispelTileset",
        "description": "Extracted tileset from .GTL or .BTL file",
        "type": "object",
        "required": ["tile_count", "tile_width", "tile_height", "color_format", "tiles"],
        "properties": {
            "tile_count": { "type": "integer" },
            "tile_width": { "type": "integer", "const": 32 },
            "tile_height": { "type": "integer", "const": 32 },
            "rendered_width": { "type": "integer", "const": 62 },
            "rendered_height": { "type": "integer", "const": 32 },
            "color_format": { "type": "string", "const": "RGB565" },
            "file_type": { "type": "string", "const": key },
            "tiles": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["index", "pixels"],
                    "properties": {
                        "index": { "type": "integer" },
                        "pixels": { "type": "null", "description": "Pixel data omitted. Use 'map tiles' to extract images." }
                    }
                }
            },
            "note": { "type": "string" }
        }
    })
}

fn infer_json_type(field: &str) -> serde_json::Value {
    let string_fields = [
        "name",
        "description",
        "filename",
        "sprite",
        "script",
        "map_filename",
        "map_name",
        "pgp_filename",
        "dlg_filename",
        "sprite_filename",
        "snf_filename",
        "title",
        "text",
        "comment",
    ];
    let boolean_fields = ["flag", "is_oversize", "is_undead", "has_blood", "lighting"];

    if string_fields.contains(&field) {
        return serde_json::json!({ "type": "string" });
    }
    if boolean_fields.contains(&field) || field.starts_with("is_") || field.starts_with("has_") {
        return serde_json::json!({ "type": "boolean" });
    }

    serde_json::json!({ "type": "integer" })
}
