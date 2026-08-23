//! Configuration setup for the crability tool. Found as a JSON in the
//! `~/.config/crability/config.json` path.

use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::status;

mod default {
    pub const CHERIBUILD_REMOTE: &str = "https://github.com/CTSRD-CHERI/cheribuild.git";
    pub const CHERIBUILD_COMMIT: &str = "8e10ca1910958b7a5cba106f0568b8d54c3f8499";
}

/// Config file location
pub fn path() -> Result<PathBuf> {
    Ok(dirs::home_dir()
        .context("Home directory could not be found")?
        .join(".config")
        .join("crability")
        .join("config.json"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoConfig {
    pub remote: String,
    pub commit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repos {
    pub cheribuild: RepoConfig,
    pub rust: RepoConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Prerequisite repos pulled into the `repos` directory
    pub repos: Repos,
    /// Morello LLVM (cloned into cheribuild's source root, not `repos`)
    pub morello_llvm: RepoConfig,
    /// Where crability keeps the pulled repos. Defaults to `~/.crability/repos`.
    repos_dir: String,
    /// Where the built compiler binaries are exposed. Defaults to `~/.crability/bin`.
    bin_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            repos: Repos {
                // TODO: Have all these values in the default module so it's easier to edit
                cheribuild: RepoConfig {
                    remote: default::CHERIBUILD_REMOTE.into(),
                    commit: default::CHERIBUILD_COMMIT.into(),
                },
                rust: RepoConfig {
                    remote: "https://github.com/irina-nita/rust.git".into(),
                    commit: "fff2dc417b563da01ef48924940d94962b268d65".into(),
                },
            },
            morello_llvm: RepoConfig {
                remote: "https://git.morello-project.org/morello/llvm-project.git".into(),
                commit: "671d6dbe2b74525702368edfa086e68f5afadc24".into(),
            },
            repos_dir: "~/.crability/repos".into(),
            bin_dir: "~/.crability/bin".into(),
        }
    }
}

impl Config {
    pub fn load_and_print(path: &Path) -> Result<Self> {
        status!(
            "Reading {} configuration from {}",
            "crability".bold().purple(),
            path.display().underline()
        );
        let config = Self::load(path)?;
        config.print();

        Ok(config)
    }

    pub fn load(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("reading config {}", path.display()))?;
        serde_json::from_str(&raw).with_context(|| format!("parsing config {}", path.display()))
    }

    fn print(&self) {
        let dirs = [("repos-dir", self.repos_dir()), ("bin-dir", self.bin_dir())];
        let repos = [
            ("cheribuild", &self.repos.cheribuild),
            ("rust", &self.repos.rust),
            ("morello-llvm", &self.morello_llvm),
        ];

        let width = dirs
            .iter()
            .map(|(name, _)| name.len())
            .chain(repos.iter().map(|(name, _)| name.len()))
            .max()
            .unwrap_or(0);

        for (name, dir) in &dirs {
            status!("  {:width$}  {}", name.bold(), dir.display().underline());
        }
        for (name, repo) in &repos {
            status!(
                "  {:width$}  {} {} {}",
                name.bold(),
                repo.remote.italic(),
                "@".dimmed(),
                short_commit(&repo.commit).yellow()
            );
        }
    }

    pub fn repos_dir(&self) -> PathBuf {
        let path = shellexpand::tilde(&self.repos_dir);
        PathBuf::from(path.as_ref())
    }

    pub fn bin_dir(&self) -> PathBuf {
        let path = shellexpand::tilde(&self.bin_dir);
        PathBuf::from(path.as_ref())
    }

    pub fn cheribuild_dir(&self) -> PathBuf {
        self.repos_dir().join("cheribuild")
    }

    pub fn rust_dir(&self) -> PathBuf {
        self.repos_dir().join("rust")
    }
}

fn short_commit(commit: &str) -> &str {
    const LEN: usize = 12;
    if commit.len() > LEN && commit.bytes().all(|b| b.is_ascii_hexdigit()) {
        &commit[..LEN]
    } else {
        commit
    }
}
