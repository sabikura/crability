use crate::context::Context;

pub(crate) mod fetch;
pub(crate) mod init;
pub(crate) mod llvm;
pub(crate) mod rust;
pub(crate) mod setup;

#[macro_export]
macro_rules! status {
    ($($arg:tt)*) => { eprintln!("  {}", format_args!($($arg)*)) };
}

#[derive(clap::Subcommand, Clone, Copy, strum::AsRefStr, strum::Display)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Command {
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

impl Command {
    pub(crate) fn run(&self) -> anyhow::Result<()> {
        let mut ctx = Context::new(*self)?;
        match self {
            Command::Init => init::run(),
            Command::Setup => setup::run(&mut ctx),
            Command::Fetch => fetch::run(&mut ctx),
            Command::Llvm => llvm::run(&mut ctx),
            Command::Rust => rust::run(&mut ctx),
        }
    }
}
