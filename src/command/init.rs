use crate::{
    config::{self, Config},
    status,
};
use anyhow::{Context as _, Result};
use owo_colors::OwoColorize;
use std::fs;

pub(crate) fn run() -> Result<()> {
    let config_path = config::path()?;
    if config_path.exists() {
        let message = format!(
            "{} configuration file found at {}. Doing nothing.",
            "crability".bold().purple(),
            config_path.display().underline()
        );
        status!("  {message}");
    } else {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
        }
        let json = serde_json::to_string_pretty(&Config::default())?;
        fs::write(&config_path, json + "\n")
            .with_context(|| format!("writing {}", config_path.display()))?;

        status!(
            "  {} configuration written at {}",
            "crability".bold().purple(),
            config_path.display().underline(),
        );
    }

    Ok(())
}
