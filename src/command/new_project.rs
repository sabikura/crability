use anyhow::{Context as _, Result, bail};
use owo_colors::OwoColorize;
use std::io::ErrorKind;
use std::process::Command;

const TEMPLATE_REPO: &str = "https://github.com/sabikura/crability-template.git";

pub(crate) fn run() -> Result<()> {
    match Command::new("cargo-generate").arg("--version").output() {
        Ok(_) => {}
        Err(err) if err.kind() == ErrorKind::NotFound => bail!(
            "cargo-generate not found. Run {} to install it",
            "cargo install cargo-generate".bold()
        ),
        Err(err) => return Err(err).context("checking for cargo-generate"),
    }

    let status = Command::new("cargo-generate")
        .args(["generate", "--git", TEMPLATE_REPO])
        .status()
        .context("running cargo-generate")?;

    if !status.success() {
        bail!("cargo-generate exited with {status}");
    }

    Ok(())
}
