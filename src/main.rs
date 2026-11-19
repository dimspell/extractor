use clap::Parser;
use dispel_core::cli::{Cli, Commands};
use dispel_core::commands::{
    Command,
    database::DatabaseCommand,
    dialog::DialogCommand,
    list::ListCommand,
    map::MapCommand,
    pack::ModPackCommand,
    schema::SchemaCommand,
    sound::SoundCommand,
    sprite::SpriteCommand,
    template::TemplateCommand,
    unified::{ExtractCommand, PatchCommand},
    validate::ValidateCommand,
};

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Some(Commands::Extract(args)) => ExtractCommand { args: args.clone() }.execute(),
        Some(Commands::Patch(args)) => PatchCommand { args: args.clone() }.execute(),
        Some(Commands::Validate(args)) => ValidateCommand { args: args.clone() }.execute(),
        Some(Commands::List(args)) => ListCommand { args: args.clone() }.execute(),
        Some(Commands::Schema(args)) => SchemaCommand { args: args.clone() }.execute(),
        Some(Commands::Template(args)) => TemplateCommand { args: args.clone() }.execute(),
        Some(Commands::Sprite {
            input,
            mode,
            no_origin_offset,
            preserve_black,
            info,
        }) => SpriteCommand {
            input: input.clone(),
            mode: *mode,
            no_origin_offset: *no_origin_offset,
            preserve_black: *preserve_black,
            info: *info,
        }
        .execute(),
        Some(Commands::Sound { command }) => SoundCommand {
            command: command.clone(),
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
        Some(Commands::Map(map_args)) => match &map_args.command {
            Some(sub) => MapCommand {
                subcommand: sub.clone(),
            }
            .execute(),
            None => {
                eprintln!("Error: 'map' requires a subcommand. Use --help for details.");
                std::process::exit(1);
            }
        },

        Some(Commands::Database(database_args)) => match &database_args.command {
            Some(sub) => DatabaseCommand {
                subcommand: sub.clone(),
            }
            .execute(),
            None => {
                eprintln!("Error: 'database' requires a subcommand. Use --help for details.");
                std::process::exit(1);
            }
        },
        Some(Commands::ModPack(args)) => ModPackCommand { args: args.clone() }.execute(),
        None => Ok(()),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
