use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use serde_json::Value;
use std::{fs, path::PathBuf};

use crate::status;

pub struct CheribuildPaths {
    pub source_root: PathBuf,
    pub output_root: PathBuf,
}

impl CheribuildPaths {
    pub fn morello_llvm_dir(&self) -> PathBuf {
        self.source_root.join("morello-llvm-project")
    }

    pub fn morello_sdk_bin(&self) -> PathBuf {
        self.output_root.join("morello-sdk").join("bin")
    }

    pub fn read() -> Result<Self> {
        let home_dir = dirs::home_dir().context("Home directory could not be found")?;
        let path = home_dir.join(".config").join("cheribuild.json");
        let mut source_root = home_dir.join("cheri");
        let mut output_root = None;

        if path.exists() {
            let raw =
                fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
            // cheribuild's JSON allows // and # comments; strip them first.
            let json: Value = serde_json::from_str(&strip_comments(&raw))
                .with_context(|| format!("parsing {}", path.display()))?;
            if let Some(s) = json.get("source-root").and_then(Value::as_str) {
                source_root = PathBuf::from(shellexpand::tilde(s).as_ref());
            }
            if let Some(o) = json.get("output-root").and_then(Value::as_str) {
                output_root = Some(PathBuf::from(shellexpand::tilde(o).as_ref()));
            }
            status!(
                "Reading {} config: {} (source-root: {})",
                "cheribuild".cyan(),
                path.display().underline(),
                source_root.display().underline()
            );
        } else {
            status!(
                "{} not found, using {} defaults (source-root: {})",
                path.display().underline(),
                "cheribuild".cyan(),
                source_root.display().underline()
            );
        }

        Ok(CheribuildPaths {
            output_root: output_root.unwrap_or_else(|| source_root.join("output")),
            source_root,
        })
    }
}

fn strip_comments(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for line in raw.lines() {
        let mut in_string = false;
        let mut prev = '\0';
        let mut cut = line.len();
        let mut chars = line.char_indices().peekable();
        while let Some((i, c)) = chars.next() {
            match c {
                '"' if prev != '\\' => in_string = !in_string,
                '#' if !in_string => {
                    cut = i;
                    break;
                }
                '/' if !in_string && matches!(chars.peek(), Some((_, '/'))) => {
                    cut = i;
                    break;
                }
                _ => {}
            }
            prev = c;
        }
        out.push_str(&line[..cut]);
        out.push('\n');
    }
    out
}
