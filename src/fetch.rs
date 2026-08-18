use crate::config::{self, Config, RepoConfig};
use anyhow::{Context, Result};
use std::{fs, path::Path, process::Command};

pub(crate) fn run() -> Result<()> {
    let config = Config::read_from_file(&config::path()?)
        .context("Could not read from config file. Please run `crability init`")?;

    let repos_dir = config.repos_dir();
    fs::create_dir_all(&repos_dir).with_context(|| format!("creating {}", repos_dir.display()))?;

    let targets = [
        (config.cheribuild_dir(), &config.repos.cheribuild),
        (config.rust_dir(), &config.repos.rust),
        (
            config.compiler_builtins_dir(),
            &config.repos.compiler_builtins,
        ),
    ];

    for (local, config) in targets {
        clone_or_update(&local, &config)?;
    }

    println!("fetch: done, repos live under {}", repos_dir.display());

    Ok(())
}

fn clone_or_update(local: &Path, config: &RepoConfig) -> Result<()> {
    if local.join(".git").exists() {
        // Check if commit exists, if not, fetch the origin
        let commit = &config.commit;
        let probe = format!("{commit}^{{commit}}");
        let commit_exists = Command::new("git")
            .args(["cat-file", "-e", &probe])
            .current_dir(local)
            .output()?
            .status
            .success();
        if !commit_exists {
            Command::new("git")
                .args(["fetch", "origin"])
                .current_dir(local)
                .status()?;
        }
    } else {
        if let Some(parent) = local.parent() {
            fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
        }
        let local_str = local.to_str().context("non-UTF-8 path")?;
        Command::new("git")
            .args(["clone", &config.remote, local_str])
            .status()?;
    }
    Command::new("git")
        .args(["checkout", "--detach", &config.commit])
        .current_dir(local)
        .status()?;

    Ok(())
}
