use super::super::snf;
use super::Command;
use std::error::Error;
use std::path::Path;

/// Sound command implementation
pub struct SoundCommand {
    pub input: String,
    pub output: String,
}

impl Command for SoundCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        println!("Extracting sound file to {}...", self.output);
        snf::extract(Path::new(&self.input), Path::new(&self.output))
            .expect("ERROR: could not convert SNF file to WAV");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "sound"
    }

    fn description(&self) -> &'static str {
        "Convert SNF audio to WAV"
    }
}
