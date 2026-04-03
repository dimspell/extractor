use super::Command;
use crate::cli::MapCommands;
use crate::map;
use std::error::Error;
use std::fs;
use std::path::Path;

pub struct MapCommand {
    pub subcommand: MapCommands,
}

impl Command for MapCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        match &self.subcommand {
            MapCommands::Tiles { input, output } => {
                eprintln!("Extracting all tiles to separate tiles...");
                eprintln!("Input file: {input:?}");
                eprintln!("Output directory: {output:?}");

                let tiles = map::tileset::extract(Path::new(input))
                    .map_err(|e| format!("ERROR: could not extract tile-set: {e}"))?;
                map::tileset::plot_all_tiles(&tiles, output);
                Ok(())
            }
            MapCommands::Atlas { input, output } => {
                eprintln!("Rendering map atlas...");
                eprintln!("Input file: {input:?}");
                eprintln!("Output file: {output:?}");

                let tiles = map::tileset::extract(Path::new(input))
                    .map_err(|e| format!("ERROR: could not extract tile-set: {e}"))?;
                map::tileset::plot_tileset_map(&tiles, output);
                Ok(())
            }
            MapCommands::Render {
                map,
                btl,
                gtl,
                output,
                save_sprites,
            } => {
                eprintln!("Rendering map...");
                map::extract(
                    Path::new(map),
                    Path::new(btl),
                    Path::new(gtl),
                    Path::new(output),
                    *save_sprites,
                )
                .map_err(|e| format!("ERROR: could not render map: {e}"))?;
                Ok(())
            }
            MapCommands::FromDb {
                database,
                map_id,
                gtl_atlas,
                btl_atlas,
                atlas_columns,
                output,
                game_path,
            } => {
                eprintln!("Rendering map from database...");
                map::render_from_database(
                    Path::new(database),
                    map_id,
                    Path::new(gtl_atlas),
                    Path::new(btl_atlas),
                    *atlas_columns,
                    Path::new(output),
                    game_path.as_deref().map(Path::new),
                )
                .map_err(|e| format!("ERROR: could not render map from database: {e}"))?;
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

    fn name(&self) -> &'static str {
        "map"
    }

    fn description(&self) -> &'static str {
        "Extract and render map assets"
    }
}
