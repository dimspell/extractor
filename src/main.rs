use clap::Parser;
use dispel_core::cli::{self, Cli};

fn main() {
    let cli = Cli::parse();

    if cli.list_commands {
        cli::print_command_spec();
        return;
    }

    // `dispatch` consumes the parsed command; argument structs are never
    // cloned or rebuilt here, and all errors (including a missing nested
    // subcommand) share one reporting path.
    if let Err(e) = cli::dispatch(cli.command) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
