use super::Command;
use crate::cli::SoundCommands;
use dispel_core::snf;
use std::error::Error;
use std::path::Path;

pub struct SoundCommand {
    pub command: SoundCommands,
}

impl Command for SoundCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        match &self.command {
            SoundCommands::ToWav { input, output } => {
                eprintln!("Extracting sound file to {}...", output);
                snf::extract(Path::new(input), Path::new(output))
                    .map_err(|e| format!("ERROR: could not convert SNF file to WAV: {e}"))?;
                Ok(())
            }
            SoundCommands::FromWav { input, output } => {
                eprintln!("Importing WAV to SNF: {} → {}", input, output);
                snf::import_wav(Path::new(input), Path::new(output))
                    .map_err(|e| format!("ERROR: could not convert WAV to SNF: {e}"))?;
                Ok(())
            }
        }
    }
}
