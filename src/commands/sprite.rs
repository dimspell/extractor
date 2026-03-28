use super::super::sprite;
use super::Command;
use std::error::Error;
use std::path::Path;

/// Sprite command implementation
pub struct SpriteCommand {
    pub input: String,
    pub mode: SpriteMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpriteMode {
    Sprite,
    Animation,
}

impl Command for SpriteCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        println!("Extracting sprite...");
        match &self.mode {
            SpriteMode::Sprite => {
                sprite::extract(&Path::new(&self.input), "todo_prefix".to_string())
                    .expect("ERROR: could not export sprite");
            }
            SpriteMode::Animation => {
                sprite::animation(&Path::new(&self.input)).expect("ERROR: could not export sprite");
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
