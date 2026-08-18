use anyhow::Result;
use clap::{Parser, Subcommand};

mod cheribuild_config;
mod config;
mod fetch;
mod init;
mod llvm;
mod rust;
mod setup;

/// crability - set up the environment for CHERI/Morello Rust development.
#[derive(Parser)]
#[command(name = "crability", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Write the default configuration to ~/.config/crability/config.json
    Init,
    /// Install the OS packages for cheribuild. Supported distributions: Debian/Ubuntu, RHEL/Fedora,
    /// Arch
    Setup,
    /// Clone (or update) cheribuild, rust and compiler-builtins at their pinned commits
    Fetch,
    /// Clone Morello LLVM into CHERI_HOME, apply the llvm.patch from the rust repo, build it
    Llvm,
    /// Generate rust/config.toml, point cheri_config.sh at CHERI_HOME and build the compiler
    Rust,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init => init::run(),
        Command::Setup => setup::run(),
        Command::Fetch => fetch::run(),
        Command::Llvm => llvm::run(),
        Command::Rust => rust::run(),
    }
}
