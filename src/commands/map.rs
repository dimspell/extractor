use super::super::map;
use super::Command;
use std::error::Error;
use std::fs;
use std::path::Path;

/// Map command implementation
pub struct MapCommand {
    pub subcommand: MapSubcommand,
}

pub enum MapSubcommand {
    Tiles {
        input: String,
        output: String,
    },
    Atlas {
        input: String,
        output: String,
    },
    Render {
        map: String,
        btl: String,
        gtl: String,
        output: String,
        save_sprites: bool,
    },
    FromDb {
        database: String,
        map_id: String,
        gtl_atlas: String,
        btl_atlas: String,
        atlas_columns: u32,
        output: String,
        game_path: Option<String>,
    },
    ToDb {
        database: String,
        map: String,
    },
    Sprites {
        input: String,
        output: String,
    },
    ToJson {
        input: String,
        output: Option<String>,
        pretty: bool,
    },
}

impl Command for MapCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        match &self.subcommand {
            MapSubcommand::Tiles { input, output } => {
                println!("Extracting all tiles to separate tiles...");
                println!("Input file: {input:?}");
                println!("Output directory: {output:?}");

                let tiles = map::tileset::extract(Path::new(input))
                    .expect("ERROR: could not extract tile-set");
                map::tileset::plot_all_tiles(&tiles, output);
                Ok(())
            }
            MapSubcommand::Atlas { input, output } => {
                println!("Rendering map atlas...");
                println!("Input file: {input:?}");
                println!("Output file: {output:?}");

                let tiles = map::tileset::extract(Path::new(input))
                    .expect("ERROR: could not extract tile-set");
                map::tileset::plot_tileset_map(&tiles, output);
                Ok(())
            }
            MapSubcommand::Render {
                map,
                btl,
                gtl,
                output,
                save_sprites,
            } => {
                println!("Rendering map...");
                map::extract(
                    Path::new(map),
                    Path::new(btl),
                    Path::new(gtl),
                    Path::new(output),
                    save_sprites,
                )
                .expect("ERROR: could not render map");
                Ok(())
            }
            MapSubcommand::FromDb {
                database,
                map_id,
                gtl_atlas,
                btl_atlas,
                atlas_columns,
                output,
                game_path,
            } => {
                println!("Rendering map from database...");
                map::render_from_database(
                    Path::new(database),
                    map_id,
                    Path::new(gtl_atlas),
                    Path::new(btl_atlas),
                    *atlas_columns,
                    Path::new(output),
                    game_path.as_deref().map(Path::new),
                )
                .expect("ERROR: could not render map from database");
                Ok(())
            }
            MapSubcommand::ToDb { database, map } => {
                println!("Importing map to database...");
                map::import_to_database(Path::new(database), Path::new(map))
                    .expect("ERROR: could not import map to database");
                Ok(())
            }
            MapSubcommand::Sprites { input, output } => {
                println!("Extracting map internal sprites to separate PNG files...");
                println!("Input file: {input:?}");
                println!("Output directory: {output:?}");

                map::extract_sprites(Path::new(input), Path::new(output))
                    .expect("ERROR: could not extract sprites");
                Ok(())
            }
            MapSubcommand::ToJson {
                input,
                output,
                pretty,
            } => {
                let file =
                    fs::File::open(input).map_err(|e| format!("Failed to open map file: {}", e))?;
                let mut reader = std::io::BufReader::new(file);
                let map_data = map::read_map_data(&mut reader)
                    .map_err(|e| format!("Failed to parse map file: {}", e))?;
                let json_data = map_data.to_json();
                let json_str = if *pretty {
                    serde_json::to_string_pretty(&json_data)
                        .map_err(|e| format!("Failed to serialize JSON: {}", e))?
                } else {
                    serde_json::to_string(&json_data)
                        .map_err(|e| format!("Failed to serialize JSON: {}", e))?
                };
                if let Some(output_path) = output {
                    fs::write(output_path, &json_str)
                        .map_err(|e| format!("Failed to write to {}: {}", output_path, e))?;
                    eprintln!("Extracted map data to {}", output_path);
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
