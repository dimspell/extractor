mod cli;
mod commands;

use clap::Parser;
use cli::{Cli, Commands};
use commands::database::DatabaseCommand;
use commands::dialog::DialogCommand;
use commands::list::ListCommand;
use commands::map::MapCommand;
use commands::pack::ModPackCommand;
use commands::schema::SchemaCommand;
use commands::sound::SoundCommand;
use commands::sprite::SpriteCommand;
use commands::template::TemplateCommand;
use commands::unified::{ExtractCommand, PatchCommand};
use commands::validate::ValidateCommand;
use commands::Command;

fn main() {
    let cli = Cli::parse();

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
            database_path,
        }) => DialogCommand {
            dlg_path: dlg_path.display().to_string(),
            pgp_path: pgp_path.as_ref().map(|p| p.display().to_string()),
            npc_ref_path: npc_ref_path.as_ref().map(|p| p.display().to_string()),
            database_path: database_path.as_ref().map(|p| p.display().to_string()),
        }
        .execute(),
        Some(Commands::Map(map_args)) => {
            match &map_args.command {
                Some(sub) => MapCommand { subcommand: sub.clone() }.execute(),
                None => {
                    eprintln!("Error: 'map' requires a subcommand. Use --help for details.");
                    std::process::exit(1);
                }
            }
        }

        Some(Commands::Database(database_args)) => {
            match &database_args.command {
                Some(sub) => DatabaseCommand { subcommand: sub.clone() }.execute(),
                None => {
                    eprintln!("Error: 'database' requires a subcommand. Use --help for details.");
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::ModPack(args)) => ModPackCommand { args: args.clone() }.execute(),
        None => Ok(()),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
