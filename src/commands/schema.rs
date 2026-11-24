use std::error::Error;

use crate::cli::SchemaArgs;
use crate::commands::Command;
use crate::commands::registry::{self, FileType};

impl Command for SchemaArgs {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        let file_type = registry::get_by_key(&self.r#type).ok_or_else(|| {
            format!(
                "Unknown file type '{}'. Available types:\n{}",
                self.r#type,
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

    let mut properties = serde_json::Map::new();
    for field in &ft.fields {
        properties.insert(
            field.name.to_string(),
            serde_json::json!({ "type": field.ty.as_schema_type() }),
        );
    }
    let required: Vec<&str> = ft.field_names().collect();

    serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": format!("{} records", ft.record_type),
        "description": ft.description,
        "type": "array",
        "items": {
            "type": "object",
            "properties": properties,
            "required": required,
        }
    })
}

fn generate_map_schema() -> serde_json::Value {
    serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "DispelMapData",
        "description": "Complete parsed representation of a Dispel game .MAP file",
        "type": "object",
        "required": ["metadata", "gtl_tiles", "btl_tiles", "collisions", "events", "object_ids", "tiled_objects", "sprites", "internal_sprites"],
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
            "object_ids": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["x", "y", "tile_id"],
                    "properties": {
                        "x": { "type": "integer", "minimum": 0 },
                        "y": { "type": "integer", "minimum": 0 },
                        "tile_id": { "type": "integer", "description": "Object id stored in the map's object-id grid" }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::{MapDataJson, MapMetadataJson};
    use std::collections::HashSet;

    #[test]
    fn test_map_schema_properties_match_serialized_map_data() {
        let schema = generate_map_schema();
        let properties = schema["properties"].as_object().expect("schema properties");
        let required = schema["required"].as_array().expect("schema required");

        let sample = MapDataJson {
            metadata: MapMetadataJson {
                chunk_width: 0,
                chunk_height: 0,
                tiled_width: 0,
                tiled_height: 0,
                map_width_in_pixels: 0,
                map_height_in_pixels: 0,
                non_occluded_start_x: 0,
                non_occluded_start_y: 0,
                occluded_width: 0,
                occluded_height: 0,
            },
            gtl_tiles: vec![],
            btl_tiles: vec![],
            collisions: vec![],
            events: vec![],
            object_ids: vec![],
            tiled_objects: vec![],
            sprites: vec![],
            internal_sprites: vec![],
        };
        let serialized = serde_json::to_value(sample).expect("serialize MapDataJson");
        let json_keys: HashSet<&String> = serialized
            .as_object()
            .expect("MapDataJson serializes to an object")
            .keys()
            .collect();
        let schema_keys: HashSet<&String> = properties.keys().collect();

        assert_eq!(
            schema_keys, json_keys,
            "schema properties must cover exactly the fields MapDataJson emits"
        );
        for key in json_keys {
            assert!(
                required.iter().any(|r| r.as_str() == Some(key.as_str())),
                "schema required list is missing {key}"
            );
        }
    }
}
