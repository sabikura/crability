use crate::command::Command;
use anyhow::{bail, Context as _, Result};
use indicatif::{ProgressBar, ProgressStyle};
use jiff::Zoned;
use owo_colors::OwoColorize;
use std::{
    collections::VecDeque,
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    path::PathBuf,
    process::{ExitStatus, Stdio},
    time::Duration,
};

pub(crate) struct Context {
    dir: PathBuf,
}

impl Context {
    pub(crate) fn new(command: &Command) -> Result<Self> {
        Context::create_dir(command).map(|dir| Self { dir })
    }

    fn create_dir(command: &Command) -> Result<PathBuf> {
        let home_dir = dirs::home_dir().context("Home directory could not be found")?;
        let root = home_dir.join(".local").join("state").join("crability");
        fs::create_dir_all(&root)?;

        let now = Zoned::now();
        let timestamp = now.strftime("%Y-%m-%dT%H-%M-%S");
        let dirname = format!("{command}-{timestamp}");
        let mut index = 0;
        loop {
            let name = if index > 0 {
                &format!("{dirname}.{index}")
            } else {
                &dirname
            };

            let path = root.join(name);
            match fs::create_dir(&path) {
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => index += 1,
                val => {
                    return val
                        .with_context(|| format!("creating {}", path.display()))
                        .map(|()| path)
                }
            }
        }
    }

    fn bar(&self) -> ProgressBar {
        let bar = ProgressBar::new_spinner();
        let style = ProgressStyle::with_template("{spinner:>2.cyan} {elapsed:.magenta} {msg}")
            .expect("invalid template");
        bar.set_style(style);

        bar
    }

    /// Run `command` as the next step of this invocation, streaming its output into
    /// `<step>.log`. Fails if the child exits non-zero, replaying the tail of the log so
    /// the error is on screen rather than only on disk.
    pub(crate) fn run(&mut self, command: &mut std::process::Command) -> Result<()> {
        let bar = self.bar();

        let command_str = command_to_str(command);

        let mut filename = command_str.replace(" ", "-");
        filename.retain(|c| !c.is_whitespace() && c != '/' && c != '\\');
        let path = self.dir.join(format!("{filename}.log"));

        let mut logfile =
            File::create(&path).with_context(|| format!("creating {}", path.display()))?;

        let command_dir = command
            .get_current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or("the current directory".to_owned());

        bar.set_message(format!(
            "Running {} (inside {})",
            command_str.italic().bold(),
            command_dir.underline()
        ));
        bar.enable_steady_tick(Duration::from_millis(100));

        let (status, stderr_tail) = write_output_to_logfile(command, &mut logfile)?;
        if !status.success() {
            let base = format!(
                "Command {} (inside {})",
                command_str.blue().bold(),
                command_dir.underline(),
            );
            let msg = match status.code() {
                Some(code) => format!("{base} exited with {}", code.red().bold()),
                None => format!("{base} terminated by signal"),
            };
            bar.finish_with_message(format!(
                "{msg}. Check full logs at {}",
                path.display().underline()
            ));
            eprintln!(
                "\n{}\n",
                stderr_tail.into_iter().collect::<Vec<_>>().join("\n").red()
            );

            bail!("Command {command_str} failed with {status}")
        } else {
            bar.finish();
            Ok(())
        }
    }
}

fn command_to_str(command: &std::process::Command) -> String {
    let argv: Vec<String> = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|s| s.to_string_lossy().to_string())
        .collect();

    argv.join(" ")
}

/// Run `command` with stdout going straight to logfile, and stderr copied into log while also
/// keeping the last 10 error logs into a _tail_ buffer so they can be printed to the user if the
/// command fails.
fn write_output_to_logfile(
    command: &mut std::process::Command,
    log: &mut File,
) -> Result<(ExitStatus, VecDeque<String>)> {
    let mut child = command
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("spawning {:?}", command.get_program()))?;

    let mut stderr = BufReader::new(child.stderr.take().expect("stderr handle already captured"));
    // Buffer to read into from stderr; used to write each line into the logfile
    let mut buf = Vec::new();
    // Fixed-size "tail" that keeps the last 10 error messages to also print to the user's stdout if
    // an error occurs
    const STDERR_TAIL: usize = 10;
    let mut tail = VecDeque::new();
    loop {
        buf.clear();
        if stderr.read_until(b'\n', &mut buf)? == 0 {
            break;
        }
        log.write_all(&buf)?;
        if tail.len() == STDERR_TAIL {
            tail.pop_front();
        }
        tail.push_back(String::from_utf8_lossy(&buf).trim_end().to_owned());
    }

    Ok((child.wait()?, tail))
}
