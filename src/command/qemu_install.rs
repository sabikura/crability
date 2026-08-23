use crate::{
    cheribuild_config::CheribuildPaths,
    command::install_bin,
    config::{self, Config},
    status,
};
use anyhow::{bail, Context as _, Result};
use owo_colors::OwoColorize;
use std::fs;

pub(crate) fn run() -> Result<()> {
    let config = Config::load_and_print(&config::path()?)
        .context("Could not read from config file. Please run `crability init`")?;

    let paths = CheribuildPaths::read()?;
    let qemu = paths.sdk_bin().join("qemu-system-morello");

    if !qemu.exists() {
        status!(
            "CHERI QEMU is not built yet ({} is missing)",
            qemu.display().underline()
        );

        let cheribuild_dir = config.cheribuild_dir();
        if !cheribuild_dir.join("cheribuild.py").exists() {
            status!("Run {} first to clone cheribuild", "crability fetch".bold());
        }
        status!(
            "Run {} inside {} to build and install it",
            "python3 cheribuild.py qemu".bold(),
            cheribuild_dir.display().underline()
        );
        status!(
            "Then re-run {} to symlink the binary",
            "crability qemu-install".bold()
        );

        bail!("CHERI QEMU has not been built by cheribuild yet");
    }

    let bin_dir = config.bin_dir();
    fs::create_dir_all(&bin_dir).with_context(|| format!("creating {}", bin_dir.display()))?;
    install_bin(&qemu, &bin_dir)?;

    status!(
        "Symlinked {} into {}",
        qemu.display().underline(),
        bin_dir.display().underline()
    );

    Ok(())
}
