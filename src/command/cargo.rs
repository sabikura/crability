use crate::config::{self, Config};
use anyhow::{bail, Context as _, Result};
use owo_colors::OwoColorize;
use std::{ffi::OsString, os::unix::process::CommandExt, process::Command};

pub(crate) fn run(args: &[OsString]) -> Result<()> {
    let config = Config::load(&config::path()?)
        .context("Could not read from config file. Please run `crability init`")?;

    let bin_dir = config.bin_dir();
    let rustc = bin_dir.join("rustc");
    if !rustc.exists() {
        bail!(
            "{} not found. Run {} to build and install the Morello toolchain",
            rustc.display(),
            "crability rust".bold()
        );
    }

    // Prefer the cargo built alongside the compiler and fall back to the host cargo with the
    // Morello rustc
    let cargo = bin_dir.join("cargo");
    let mut command = if cargo.exists() {
        Command::new(cargo)
    } else {
        Command::new("cargo")
    };
    command.args(args).env("RUSTC", &rustc);

    let err = command.exec();
    Err(err).context("running cargo")
}
