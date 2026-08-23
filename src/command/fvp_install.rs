use crate::{
    cheribuild_config::CheribuildPaths,
    command::install_bin,
    config::{self, Config},
    status,
};
use anyhow::{bail, Context as _, Result};
use owo_colors::OwoColorize;
use std::{fs, path::Path};

pub(crate) fn run() -> Result<()> {
    let config = Config::load_and_print(&config::path()?)
        .context("Could not read from config file. Please run `crability init`")?;

    let paths = CheribuildPaths::read()?;
    let model = paths
        .morello_fvp_dir()
        .join("models")
        .join("Linux64_GCC-6.4")
        .join("FVP_Morello");

    if !model.exists() {
        explain_cheribuild_step(&config, &model);
        bail!("The Morello FVP has not been installed by cheribuild yet");
    }

    let bin_dir = config.bin_dir();
    fs::create_dir_all(&bin_dir).with_context(|| format!("creating {}", bin_dir.display()))?;
    install_bin(&model, &bin_dir)?;

    status!(
        "Symlinked {} into {}",
        model.display().underline(),
        bin_dir.display().underline()
    );

    Ok(())
}

/// The FVP ships under an Arm EULA that its installer shows interactively, so the download and
/// install stay a manual cheribuild step run by the user in their own terminal.
fn explain_cheribuild_step(config: &Config, model: &Path) {
    status!(
        "The Morello FVP is not installed yet ({} is missing)",
        model.display().underline()
    );

    let cheribuild_dir = config.cheribuild_dir();
    if !cheribuild_dir.join("cheribuild.py").exists() {
        status!("Run {} first to clone cheribuild", "crability fetch".bold());
    }
    status!(
        "Run {} inside {} to download and install it (the installer asks to accept the EULA)",
        "python3 cheribuild.py install-morello-fvp".bold(),
        cheribuild_dir.display().underline()
    );
    status!(
        "An already-downloaded installer can be passed with {}",
        "--install-morello-fvp/installer-path <file>".bold()
    );
    status!(
        "Then re-run {} to symlink the model binary",
        "crability fvp-install".bold()
    );
}
