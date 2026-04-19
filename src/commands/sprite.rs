use super::Command;
use crate::cli::SpriteMode as CliSpriteMode;
use dispel_core::sprite;
use std::error::Error;
use std::path::Path;

/// Sprite command implementation
pub struct SpriteCommand {
    pub input: String,
    pub mode: CliSpriteMode,
    pub info: bool,
}

/// Re-export for main.rs dispatch
pub use crate::cli::SpriteMode;

impl Command for SpriteCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        if self.info {
            let info = sprite::get_sprite_info(Path::new(&self.input))
                .map_err(|e| format!("ERROR: could not read sprite info: {e}"))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&info)
                    .map_err(|e| format!("ERROR: could not encode JSON: {e}"))?
            );
            return Ok(());
        }

        eprintln!("Extracting sprite...");
        match &self.mode {
            SpriteMode::Sprite => {
                let prefix = Path::new(&self.input)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("sprite");
                sprite::extract(Path::new(&self.input), prefix.to_string())
                    .map_err(|e| format!("ERROR: could not export sprite: {e}"))?;
            }
            SpriteMode::Animation => {
                sprite::animation(Path::new(&self.input))
                    .map_err(|e| format!("ERROR: could not export sprite: {e}"))?;
            }
        }
        Ok(())
    }
}
