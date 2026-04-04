// Command module structure

pub mod database;
pub mod list;
pub mod map;
pub mod ref_command;
pub mod registry;
pub mod schema;
pub mod sound;
pub mod sprite;
pub mod template;
pub mod test;
pub mod unified;
pub mod validate;

use std::error::Error;

/// Command trait - all CLI commands implement this.
pub trait Command: Send + Sync {
    fn execute(&self) -> Result<(), Box<dyn Error>>;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
}

// ===========================================================================
// Compatibility shims for dispel-gui
// ===========================================================================

/// Empty service container (no longer used, kept for GUI compatibility).
pub struct ServiceContainer;

impl ServiceContainer {
    pub fn new() -> Self {
        ServiceContainer
    }
}

/// Command factory (no longer needed, kept for GUI compatibility).
/// All commands can now be constructed directly.
pub struct CommandFactory;

impl CommandFactory {
    pub fn new() -> Self {
        CommandFactory
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

    pub fn create_sprite_command(
        &self,
        input: String,
        mode: sprite::SpriteMode,
        info: bool,
    ) -> impl Command {
        sprite::SpriteCommand { input, mode, info }
    }

    pub fn create_sound_command(&self, input: String, output: String) -> impl Command {
        sound::SoundCommand { input, output }
    }

    pub fn create_test_command(&self, message: String) -> impl Command {
        test::TestCommand { message }
    }
}
