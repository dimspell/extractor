use super::{Command, missing_subcommand};
use crate::cli::{MapArgs, MapCommands};
use crate::map;
use std::error::Error;
use std::fs;
use std::path::Path;

impl Command for MapArgs {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        match &self.command {
            Some(command) => command.execute(),
            None => Err(missing_subcommand("map")),
        }
    }
}

impl Command for MapCommands {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        match self {
            MapCommands::Tiles { input, output } => {
                eprintln!("Extracting all tiles to separate tiles...");
                eprintln!("Input file: {input:?}");
                eprintln!("Output directory: {output:?}");

                let tiles = map::tileset::extract(Path::new(input))
                    .map_err(|e| format!("ERROR: could not extract tile-set: {e}"))?;
                map::tileset::plot_all_tiles(&tiles, output)?;
                Ok(())
            }
            MapCommands::Atlas {
                input,
                output,
                opaque,
            } => {
                eprintln!("Rendering map atlas...");
                eprintln!("Input file: {input:?}");
                eprintln!("Output file: {output:?}");

                let tiles = map::tileset::extract(Path::new(input))
                    .map_err(|e| format!("ERROR: could not extract tile-set: {e}"))?;
                // The ground pass copies the tile rows opaquely; the BTL pass
                // skips the black pixels of the shadowed refs whose per-ref
                // blend byte is non-zero.
                let is_gtl = input.to_ascii_lowercase().ends_with(".gtl");
                let opaque = opaque.unwrap_or(is_gtl);
                eprintln!(
                    "Atlas mode: {}",
                    if opaque {
                        "opaque (ground row-copy)"
                    } else {
                        "skip-zero (BTL blend)"
                    }
                );
                map::tileset::plot_tileset_map_with(&tiles, output, opaque)?;
                Ok(())
            }
            MapCommands::Render {
                map,
                btl,
                gtl,
                output,
                save_sprites,
                game_path,
                no_ground,
                no_buildings,
                no_roofs,
                no_shadows,
                light_template,
                camera,
                no_internal_sprites,
                no_monsters,
                no_npcs,
                no_objects,
                frame,
                transparent,
                collisions,
                events,
                draw_items,
                npc_waypoints,
            } => {
                eprintln!("Rendering map...");
                if let Some(template_path) = light_template.as_deref() {
                    eprintln!("Applying light template {template_path}...");
                }
                map::extract(
                    Path::new(map),
                    Path::new(btl),
                    Path::new(gtl),
                    Path::new(output),
                    *save_sprites,
                    game_path.as_deref().map(Path::new),
                    map::render::LayerToggles {
                        show_ground: !no_ground,
                        show_buildings: !no_buildings,
                        show_roofs: !no_roofs,
                        show_shadows: !no_shadows,
                        show_internal_sprites: !no_internal_sprites,
                        show_monsters: !no_monsters,
                        show_npcs: !no_npcs,
                        show_objects: !no_objects,
                        transparent: *transparent,
                        show_collisions: *collisions,
                        show_events: *events,
                        show_draw_items: *draw_items,
                        show_npc_waypoints: *npc_waypoints,
                        sprite_frame: *frame,
                    },
                    light_template
                        .as_deref()
                        .map(Path::new)
                        .map(|p| {
                            map::light_template::LightTemplate::load(p).map_err(|e| {
                                format!("ERROR: could not load light template {p:?}: {e}")
                            })
                        })
                        .transpose()?,
                    camera.as_deref(),
                )
                .map_err(|e| format!("ERROR: could not render map: {e}"))?;
                Ok(())
            }
            MapCommands::ToDb { database, map } => {
                eprintln!("Importing map to database...");
                map::import_to_database(Path::new(database), Path::new(map))
                    .map_err(|e| format!("ERROR: could not import map to database: {e}"))?;
                Ok(())
            }
            MapCommands::Sprites { input, output } => {
                eprintln!("Extracting map internal sprites to separate PNG files...");
                eprintln!("Input file: {input:?}");
                eprintln!("Output directory: {output:?}");

                map::extract_sprites(Path::new(input), Path::new(output))
                    .map_err(|e| format!("ERROR: could not extract sprites: {e}"))?;
                Ok(())
            }
            MapCommands::Tmx {
                map,
                gtl,
                btl,
                output,
            } => {
                eprintln!("Exporting map to TMX...");
                use std::fs::File;
                use std::io::BufReader;
                let file = File::open(map).map_err(|e| format!("Failed to open map file: {e}"))?;
                let mut reader = BufReader::new(file);
                let map_data = crate::map::read_map_data(&mut reader)
                    .map_err(|e| format!("Failed to parse map: {e}"))?;
                let output_dir = std::path::Path::new(output);
                std::fs::create_dir_all(output_dir)
                    .map_err(|e| format!("Failed to create output directory: {e}"))?;
                crate::map::tmx::export_tmx(
                    &map_data,
                    std::path::Path::new(gtl),
                    std::path::Path::new(btl),
                    output_dir,
                )
                .map_err(|e| format!("TMX export failed: {e}"))?;
                eprintln!("TMX export complete: {output}");
                Ok(())
            }
            MapCommands::ToJson {
                input,
                output,
                pretty,
            } => {
                let file =
                    fs::File::open(input).map_err(|e| format!("Failed to open map file: {e}"))?;
                let mut reader = std::io::BufReader::new(file);
                let map_data = map::read_map_data(&mut reader)
                    .map_err(|e| format!("Failed to parse map file: {e}"))?;
                let json_data = map_data.to_json();
                let json_str = if *pretty {
                    serde_json::to_string_pretty(&json_data)
                        .map_err(|e| format!("Failed to serialize JSON: {e}"))?
                } else {
                    serde_json::to_string(&json_data)
                        .map_err(|e| format!("Failed to serialize JSON: {e}"))?
                };
                if let Some(output_path) = output {
                    fs::write(output_path, &json_str)
                        .map_err(|e| format!("Failed to write to {output_path}: {e}"))?;
                    eprintln!("Extracted map data to {output_path}");
                } else {
                    println!("{}", json_str);
                }
                Ok(())
            }
        }
    }
}
