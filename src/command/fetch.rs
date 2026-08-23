use crate::{
    config::{self, Config, RepoConfig},
    context::Context,
};
use anyhow::{Context as _, Result};
use owo_colors::OwoColorize;
use std::{fs, path::Path, process::Command};

pub(crate) fn run(ctx: &mut Context) -> Result<()> {
    let config = Config::load_and_print(&config::path()?)
        .context("Could not read from config file. Please run `crability init`")?;

    let repos_dir = config.repos_dir();
    fs::create_dir_all(&repos_dir).with_context(|| format!("creating {}", repos_dir.display()))?;

    let targets = [
        (config.cheribuild_dir(), &config.repos.cheribuild),
        (config.rust_dir(), &config.repos.rust),
    ];

    for (local, config) in targets {
        clone_or_update(&local, &config, ctx)?;
    }

    eprintln!(
        "Fetched repositories can be found under {}",
        repos_dir.display().underline()
    );

    Ok(())
}

fn clone_or_update(local: &Path, config: &RepoConfig, ctx: &mut Context) -> Result<()> {
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
            ctx.run(
                Command::new("git")
                    .args(["fetch", "origin"])
                    .current_dir(local),
            )?;
        }
    } else {
        if let Some(parent) = local.parent() {
            fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
        }
        let local_str = local.to_str().context("non-UTF-8 path")?;
        let mut command = Command::new("git");
        command.args(["clone", &config.remote, local_str]);

        ctx.run(&mut command)?;
    }

    ctx.run(
        Command::new("git")
            .args(["checkout", "--detach", &config.commit])
            .current_dir(local),
    )?;

    Ok(())
}
