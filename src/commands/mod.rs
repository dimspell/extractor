// Command module structure
// This will contain the Command trait and command implementations

pub mod database;
pub mod info;
pub mod map;
pub mod ref_command;
pub mod registry;
pub mod sound;
pub mod sprite;
pub mod test;
pub mod unified;

use std::error::Error;

/// Command trait - all CLI commands implement this.
pub trait Command: Send + Sync {
    fn execute(&self) -> Result<(), Box<dyn Error>>;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
}
