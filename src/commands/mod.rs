// Command module structure

pub mod database;
pub mod dialog;
pub mod list;
pub mod map;
pub mod pack;
pub mod registry;
pub mod schema;
pub mod sound;
pub mod sprite;
pub mod template;
pub mod unified;
pub mod validate;

use std::error::Error;

/// Command trait - all CLI commands implement this.
pub trait Command: Send + Sync {
    fn execute(&self) -> Result<(), Box<dyn Error>>;
}

/// Shared error for a parent command invoked without its required subcommand.
/// Returned from `execute()` so `main` reports it like any other command error
/// instead of `eprintln!`-ing and exiting from inside the dispatch match.
pub fn missing_subcommand(parent: &str) -> Box<dyn Error> {
    format!("'{parent}' requires a subcommand. Use --help for details.").into()
}
