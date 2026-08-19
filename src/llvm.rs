use crate::{
    cheribuild_config::CheribuildPaths,
    config::{self, Config, RepoConfig},
};
use anyhow::{bail, Context, Ok, Result};
use std::{fs, path::Path, process::Command};

pub(crate) fn run() -> Result<()> {
    let paths = CheribuildPaths::read()?;
    let config = Config::read_from_file(&config::path()?)
        .context("Could not read from config file. Please run `crability init`")?;

    fs::create_dir_all(&paths.source_root)
        .with_context(|| format!("creating {}", paths.source_root.display()))?;

    let llvm_dir = paths.morello_llvm_dir();
    let freshly_cloned = !llvm_dir.join(".git").exists();
    clone_morello_llvm(&llvm_dir, &config.morello_llvm)?;

    // Apply the patch shipped in the rust repo.
    let patch = config.rust_dir().join("llvm.patch");
    if !patch.exists() {
        bail!("{} not found. Run `crability fetch` first", patch.display());
    }

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
        Command::new("git")
            .current_dir(llvm_dir)
            .args(["apply", patch_str])
            .status()?;
    }

    // Build via cheribuild
    Command::new("python3")
        .current_dir(config.cheribuild_dir())
        .args(["cheribuild.py", "--skip-update", "morello-llvm"])
        .status()?;

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
fn clone_morello_llvm(local: &Path, config: &RepoConfig) -> Result<()> {
    if local.join(".git").exists() {
        println!("{} already cloned, skipping", local.display());
        return Ok(());
    }
    if let Some(parent) = local.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    let local_str = local.to_str().context("non-UTF-8 path")?;

    Command::new("git")
        .args([
            "clone",
            "--revision",
            &config.commit,
            "--depth",
            "1",
            &config.remote,
            local_str,
        ])
        .status()?;

    Ok(())
}
