use super::super::sprite;
use super::Command;
use std::error::Error;
use std::path::Path;

/// Sprite command implementation
pub struct SpriteCommand {
    pub input: String,
    pub mode: SpriteMode,
    pub info: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpriteMode {
    Sprite,
    Animation,
}

impl Command for SpriteCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        if self.info {
            let info = sprite::get_sprite_info(Path::new(&self.input))
                .expect("ERROR: could not read sprite info");
            println!(
                "{}",
                serde_json::to_string_pretty(&info).expect("ERROR: could not encode JSON")
            );
            return Ok(());
        }

        println!("Extracting sprite...");
        match &self.mode {
            SpriteMode::Sprite => {
                let prefix = Path::new(&self.input)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("sprite");
                sprite::extract(Path::new(&self.input), prefix.to_string())
                    .expect("ERROR: could not export sprite");
            }
            SpriteMode::Animation => {
                sprite::animation(Path::new(&self.input)).expect("ERROR: could not export sprite");
            }
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "sprite"
    }

    fn description(&self) -> &'static str {
        "Extract frames or sequences from SPR files"
    }
}
