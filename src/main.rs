use clap::Parser;
use dispel_core::cli::{Cli, Commands};
use dispel_core::commands::database::DatabaseCommand;
use dispel_core::commands::list::ListCommand;
use dispel_core::commands::map::MapCommand;
use dispel_core::commands::ref_command::RefCommand;
use dispel_core::commands::schema::SchemaCommand;
use dispel_core::commands::sound::SoundCommand;
use dispel_core::commands::sprite::SpriteCommand;
use dispel_core::commands::template::TemplateCommand;
use dispel_core::commands::test::TestCommand;
use dispel_core::commands::unified::{ExtractCommand, PatchCommand};
use dispel_core::commands::validate::ValidateCommand;
use dispel_core::commands::Command;

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
        Some(Commands::Ref(ref_args)) => {
            if let Some(ref_command) = &ref_args.command {
                eprintln!("Note: 'ref' command is deprecated. Use 'extract' instead:");
                eprintln!(
                    "  dispel-extractor extract --input <file> --type {}",
                    ref_command.extract_type_key()
                );
                RefCommand {
                    subcommand: ref_command.clone(),
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
