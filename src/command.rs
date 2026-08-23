use crate::context::Context;
use anyhow::{Context as _, Result};
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

pub(crate) mod cargo;
pub(crate) mod fetch;
pub(crate) mod fvp_install;
pub(crate) mod init;
pub(crate) mod llvm;
pub(crate) mod qemu_install;
pub(crate) mod rust;
pub(crate) mod setup;

#[macro_export]
macro_rules! status {
    ($($arg:tt)*) => { eprintln!("  {}", format_args!($($arg)*)) };
}

#[derive(clap::Subcommand, Clone, strum::AsRefStr, strum::Display)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Command {
    /// Write the default configuration to ~/.config/crability/config.json
    Init,
    /// Install the OS packages for cheribuild. Supported distributions: Debian/Ubuntu, RHEL/Fedora,
    /// Arch
    Setup,
    /// Clone (or update) cheribuild and rust at their pinned commits
    Fetch,
    /// Clone Morello LLVM into CHERI_HOME, apply the llvm.patch from the rust repo, build it
    Llvm,
    /// Generate rust/config.toml, point cheri_config.sh at CHERI_HOME and build the compiler
    Rust,
    /// Symlink the cheribuild-installed `Morello FVP` into the crability bin dir
    FvpInstall,
    /// Symlink the cheribuild-built `qemu-system-morello` into the crability bin dir
    QemuInstall,
    /// Run cargo with the Morello Rust toolchain built by `crability rust`
    Cargo {
        /// Arguments passed through to cargo
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<OsString>,
    },
}

impl Command {
    pub(crate) fn run(&self) -> anyhow::Result<()> {
        match self {
            Command::Init => init::run(),
            Command::Setup => setup::run(&mut Context::new(self)?),
            Command::Fetch => fetch::run(&mut Context::new(self)?),
            Command::Llvm => llvm::run(&mut Context::new(self)?),
            Command::Rust => rust::run(&mut Context::new(self)?),
            Command::FvpInstall => fvp_install::run(),
            Command::QemuInstall => qemu_install::run(),
            Command::Cargo { args } => cargo::run(args),
        }
    }
}

/// Link every file in `src_dir` into `bin_dir`
pub(crate) fn install_bins_from(src_dir: &Path, bin_dir: &Path) -> Result<()> {
    fs::create_dir_all(bin_dir).with_context(|| format!("creating {}", bin_dir.display()))?;

    let entries =
        fs::read_dir(src_dir).with_context(|| format!("reading {}", src_dir.display()))?;
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            continue;
        }

        install_bin(&path, bin_dir)?;
    }

    Ok(())
}

/// Create a link to `path` inside `bin_dir`
pub(crate) fn install_bin(path: &Path, bin_dir: &Path) -> Result<()> {
    let Some(name) = path.file_name() else {
        return Ok(());
    };

    let dest = bin_dir.join(name);

    // This doesnt follow the symlink. Just check the file is fine before trying to delete it
    // TODO: Log files with issues
    if fs::symlink_metadata(&dest).is_ok() {
        fs::remove_file(&dest).with_context(|| format!("replacing {}", dest.display()))?;
    }

    symlink(path, &dest)
        .with_context(|| format!("linking {} -> {}", dest.display(), path.display()))
}
