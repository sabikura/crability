use clap::Parser;
use command::Command;
use owo_colors::OwoColorize as _;

mod cheribuild_config;
mod command;
mod config;
mod context;

/// crability - set up the environment for CHERI/Morello Rust development.
#[derive(clap::Parser)]
#[command(name = "crability", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = cli.command.run() {
        status!("{}", "Encountered an error :(".red());
        status!("{err}");
    }
}
