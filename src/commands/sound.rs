use super::Command;
use dispel_core::snf;
use std::error::Error;
use std::path::Path;

/// Sound command implementation
pub struct SoundCommand {
    pub input: String,
    pub output: String,
}

impl Command for SoundCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        eprintln!("Extracting sound file to {}...", self.output);
        snf::extract(Path::new(&self.input), Path::new(&self.output))
            .map_err(|e| format!("ERROR: could not convert SNF file to WAV: {e}"))?;
        Ok(())
    }
}
