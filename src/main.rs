mod cli;
mod commands;

use clap::Parser;
use cli::{Cli, Commands};
use commands::database::DatabaseCommand;
use commands::dialog::DialogCommand;
use commands::list::ListCommand;
use commands::map::MapCommand;
use commands::schema::SchemaCommand;
use commands::sound::SoundCommand;
use commands::sprite::SpriteCommand;
use commands::template::TemplateCommand;
use commands::test::TestCommand;
use commands::unified::{ExtractCommand, PatchCommand};
use commands::validate::ValidateCommand;
use commands::Command;

fn main() {
    let cli = Cli::parse();

    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {name}");
    }

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    let result = match &cli.command {
        Some(Commands::Extract(args)) => ExtractCommand { args: args.clone() }.execute(),
        Some(Commands::Patch(args)) => PatchCommand { args: args.clone() }.execute(),
        Some(Commands::Validate(args)) => ValidateCommand { args: args.clone() }.execute(),
        Some(Commands::List(args)) => ListCommand { args: args.clone() }.execute(),
        Some(Commands::Schema(args)) => SchemaCommand { args: args.clone() }.execute(),
        Some(Commands::Template(args)) => TemplateCommand { args: args.clone() }.execute(),
        Some(Commands::Sprite { input, mode, info }) => SpriteCommand {
            input: input.clone(),
            mode: *mode,
            info: *info,
        }
        .execute(),
        Some(Commands::Sound { input, output }) => SoundCommand {
            input: input.clone(),
            output: output.clone(),
        }
        .execute(),
        Some(Commands::Dialog {
            dlg_path,
            pgp_path,
            npc_ref_path,
        }) => DialogCommand {
            dlg_path: dlg_path.display().to_string(),
            pgp_path: pgp_path.as_ref().map(|p| p.display().to_string()),
            npc_ref_path: npc_ref_path.as_ref().map(|p| p.display().to_string()),
        }
        .execute(),
        Some(Commands::Map(map_args)) => {
            if let Some(map_command) = &map_args.command {
                MapCommand {
                    subcommand: map_command.clone(),
                }
                .execute()
            } else {
                Ok(())
            }
        }

        Some(Commands::Database(database_args)) => {
            if let Some(database_command) = &database_args.command {
                DatabaseCommand {
                    subcommand: database_command.clone(),
                }
                .execute()
            } else {
                Ok(())
            }
        }
        Some(Commands::Test { message }) => TestCommand {
            message: message.clone(),
        }
        .execute(),
        None => Ok(()),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
