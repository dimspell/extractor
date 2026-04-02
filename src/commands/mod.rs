// Command module structure
// This will contain the Command trait and command implementations

pub mod database;
pub mod map;
pub mod ref_command;
pub mod services;
pub mod sound;
pub mod sprite;
pub mod test;

use std::error::Error;

/// Command trait defining the interface for all CLI commands
pub trait Command: Send + Sync {
    /// Execute the command
    fn execute(&self) -> Result<(), Box<dyn Error>>;

    #[allow(dead_code)]
    /// Get command name
    fn name(&self) -> &'static str;

    #[allow(dead_code)]
    /// Get command description
    fn description(&self) -> &'static str;
}

/// Command factory for creating commands with dependency injection
pub struct CommandFactory {
    _services: services::ServiceContainer,
}

impl Default for CommandFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandFactory {
    pub fn new() -> Self {
        CommandFactory {
            _services: services::ServiceContainer::new(),
        }
    }

    pub fn create_map_command(&self, subcommand: map::MapSubcommand) -> impl Command {
        map::MapCommand { subcommand }
    }

    pub fn create_ref_command(&self, subcommand: ref_command::RefSubcommand) -> impl Command {
        ref_command::RefCommand { subcommand }
    }

    pub fn create_database_command(
        &self,
        subcommand: database::DatabaseSubcommand,
    ) -> impl Command {
        database::DatabaseCommand { subcommand }
    }

    pub fn create_sprite_command(&self, input: String, mode: sprite::SpriteMode) -> impl Command {
        sprite::SpriteCommand { input, mode }
    }

    pub fn create_sound_command(&self, input: String, output: String) -> impl Command {
        sound::SoundCommand { input, output }
    }

    pub fn create_test_command(&self, message: String) -> impl Command {
        test::TestCommand { message }
    }
}
