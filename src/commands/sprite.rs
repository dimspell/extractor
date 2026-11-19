use super::Command;
use crate::cli::{SpriteArgs, SpriteMode};
use crate::sprite;
use std::error::Error;
use std::path::Path;

impl Command for SpriteArgs {
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
                sprite::extract_with_options(
                    Path::new(&self.input),
                    prefix.to_string(),
                    sprite::SpriteRenderOptions {
                        preserve_black: self.preserve_black,
                        ..Default::default()
                    },
                )
                .map_err(|e| format!("ERROR: could not export sprite: {e}"))?;
            }
            SpriteMode::Animation => {
                sprite::animation_with_options(
                    Path::new(&self.input),
                    sprite::SpriteRenderOptions {
                        preserve_black: self.preserve_black,
                        apply_origin_offsets: !self.no_origin_offset,
                    },
                )
                .map_err(|e| format!("ERROR: could not export sprite: {e}"))?;
            }
        }
        Ok(())
    }
}
