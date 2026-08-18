use crate::config::{self, Config};
use anyhow::{Context, Result};
use indicatif::ProgressBar;
use owo_colors::OwoColorize;
use std::fs;

pub(crate) fn run() -> Result<()> {
    let config_path = config::path()?;

    let bar = ProgressBar::new_spinner();
    bar.set_message(format!(
        "Initialising the configuration file at path {}",
        config_path.display().underline().blue(),
    ));

    if config_path.exists() {
        bar.finish_with_message(format!(
            "{} configuration file found at {}. Doing nothing.",
            "crability".bold().purple(),
            config_path.display().underline().red()
        ));
    } else {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
        }
        let json = serde_json::to_string_pretty(&Config::default())?;
        fs::write(&config_path, json + "\n")
            .with_context(|| format!("writing {}", config_path.display()))?;

        bar.finish_with_message(format!(
            "{} configuration written at {}",
            "crability".bold().purple(),
            config_path.display().underline().green()
        ));
    }

    Ok(())
}
