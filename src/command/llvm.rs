use crate::{
    cheribuild_config::CheribuildPaths,
    config::{self, Config, RepoConfig},
    context::Context,
    status,
};
use anyhow::{bail, Context as _, Result};
use owo_colors::OwoColorize;
use std::{fs, path::Path, process::Command};

pub(crate) fn run(ctx: &mut Context) -> Result<()> {
    let paths = CheribuildPaths::read()?;
    let config = Config::load_and_print(&config::path()?)
        .context("Could not read from config file. Please run `crability init`")?;

    fs::create_dir_all(&paths.source_root)
        .with_context(|| format!("creating {}", paths.source_root.display()))?;

    let patch = config.rust_dir().join("llvm.patch");
    if !patch.exists() {
        bail!(
            "LLVM patch {} not found. Run {} first",
            patch.display().underline(),
            "crability fetch".bold()
        );
    }

    let llvm_dir = paths.morello_llvm_dir();
    let freshly_cloned = !llvm_dir.join(".git").exists();
    clone_morello_llvm(&llvm_dir, &config.morello_llvm, ctx)?;

    // Apply the patch shipped in the rust repo.
    let mut applied = false;
    let patch_str = patch.to_str().context("non-UTF-8 path")?;
    if !freshly_cloned {
        applied = std::process::Command::new("git")
            .args(["apply", "--reverse", "--check", patch_str])
            .current_dir(&llvm_dir)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
    }

    if !applied {
        ctx.run(
            Command::new("git")
                .current_dir(llvm_dir)
                .args(["apply", patch_str]),
        )?;
    }

    // Build via cheribuild
    ctx.run(
        Command::new("python3")
            .current_dir(config.cheribuild_dir())
            .args(["cheribuild.py", "--skip-update", "morello-llvm"]),
    )?;

    let llvm_config = paths.morello_sdk_bin().join("llvm-config");
    if !llvm_config.exists() {
        bail!(
            "cheribuild reported success but {} is missing. \
             Check cheribuild's output-root setting",
            llvm_config.display()
        );
    }

    Ok(())
}

/// Morello LLVM lives in cheribuild's source root rather than in `repos_dir`, and it is big
/// enough that we only ever clone it once, at the pinned commit.
fn clone_morello_llvm(local: &Path, config: &RepoConfig, ctx: &mut Context) -> Result<()> {
    if local.join(".git").exists() {
        let local = local.display();
        status!(
            "Morello LLVM {} already cloned, skipping",
            local.underline()
        );
        return Ok(());
    }
    if let Some(parent) = local.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    let local_str = local.to_str().context("non-UTF-8 path")?;

    ctx.run(Command::new("git").args([
        "clone",
        "--revision",
        &config.commit,
        "--depth",
        "1",
        &config.remote,
        local_str,
    ]))
}
