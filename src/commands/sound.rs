use super::{Command, missing_subcommand};
use crate::cli::{SoundArgs, SoundCommands};
use crate::snf;
use std::error::Error;
use std::path::Path;

impl Command for SoundArgs {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        match &self.command {
            Some(command) => command.execute(),
            None => Err(missing_subcommand("sound")),
        }
    }
}

impl Command for SoundCommands {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        match self {
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
