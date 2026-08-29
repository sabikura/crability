use crate::context::Context;
use crate::status;
use anyhow::{Context as _, Result};
use anyhow::{anyhow, bail};
use nix::unistd::Uid;
use owo_colors::OwoColorize;
use std::process::Command;

/// Where we got the needed packages list from
/// TODO: Fix this to the commit in the config json
const _PRE_BUILD_SETUP_URL: &str = "https://github.com/CTSRD-CHERI/cheribuild#pre-build-setup";

/// Supported platforms for the setup step
/// TODO: Also support macOS
#[derive(Debug, strum::Display)]
#[strum(serialize_all = "kebab-case")]
enum Platform {
    Debian,
    Fedora,
    Arch,
}

pub(crate) fn run(ctx: &mut Context) -> Result<()> {
    let platform = match std::env::consts::OS {
        "linux" => {
            let os_release = rs_release::parse_os_release("/etc/os-release")?;
            let id = os_release
                .get("ID")
                .or_else(|| os_release.get("ID_LIKE"))
                .with_context(|| "Failed to parse os-release to get information about the current Linux distribution")?
                .to_ascii_lowercase();

            match id.trim_matches('"') {
                "arch" => Ok(Platform::Arch),
                "fedora" | "centos" | "rhel" => Ok(Platform::Fedora),
                "debian" | "ubuntu" => Ok(Platform::Debian),
                p => Err(anyhow!(
                    "Operating System Id {} not yet supported for this step",
                    p
                )),
            }
        }
        p => Err(anyhow!(
            "Operating System {} not yet supported for this step",
            p
        )),
    }?;

    status!(
        "Installing {} prerequisites for the current platform: {}",
        "cheribuild".bold().purple(),
        platform.blue().underline()
    );

    refresh_sudo_credentials()?;

    let mut command = match platform {
        Platform::Debian => install_debian_command(),
        Platform::Fedora => install_fedora_command(),
        Platform::Arch => install_arch_command(),
    };

    ctx.run(&mut command)
}

/// Prompt for the sudo password up front, while we still have the terminal to ourselves.
///
/// The install step itself runs under [`Context::run`], which redraws a spinner over stderr ten
/// times a second. sudo's prompt goes to the tty and gets painted over immediately.
fn refresh_sudo_credentials() -> Result<()> {
    // TODO: Maybe move this into `context` to run a command without progress bar?
    if Uid::effective().is_root() {
        return Ok(());
    }

    let mut command = Command::new("sudo");
    command.arg("--validate");
    let status = command.status()?;

    if !status.success() {
        let command_str = "sudo --validate".bold();
        let msg = match status.code() {
            Some(code) => format!("{command_str} exited with {}", code.red().bold()),
            None => format!("{command_str} terminated by signal"),
        };
        status!("{msg}");

        bail!("{command_str} failed with {status}")
    } else {
        Ok(())
    }
}

fn privileged_command(program: &str, args: &[&str]) -> Command {
    let mut argv = Vec::with_capacity(args.len() + 2);
    if !Uid::effective().is_root() {
        // TODO: Maybe check for doas also?
        argv.push("sudo");
    }
    argv.push(program);
    argv.extend_from_slice(args);

    let mut command = Command::new(argv[0]);
    command.args(&argv[1..]);
    command
}

fn install_debian_command() -> Command {
    privileged_command(
        "apt",
        &[
            "install",
            "-y",
            "autoconf",
            "automake",
            "libtool",
            "pkg-config",
            "clang",
            "bison",
            "cmake",
            "mercurial",
            "ninja-build",
            "samba",
            "flex",
            "texinfo",
            "time",
            "libglib2.0-dev",
            "libpixman-1-dev",
            "libarchive-dev",
            "libarchive-tools",
            "libbz2-dev",
            "libattr1-dev",
            "libcap-ng-dev",
            "libexpat1-dev",
            "libgmp-dev",
            "bc",
            "tzdata",
            // Host compiler for the cheribuild steps (see `find_host_gcc`)
            "gcc-14",
            "g++-14",
        ],
    )
}

fn install_fedora_command() -> Command {
    privileged_command(
        "dnf",
        &[
            "install",
            "-y",
            "libtool",
            "clang-devel",
            "bison",
            "cmake",
            "mercurial",
            "ninja-build",
            "samba",
            "flex",
            "texinfo",
            "glib2-devel",
            "pixman-devel",
            "libarchive-devel",
            "bsdtar",
            "bzip2-devel",
            "libattr-devel",
            "libcap-ng-devel",
            "expat-devel",
            "time",
            // Host compiler for the cheribuild steps (see `find_host_gcc`):
            // gcc14/gcc14-c++ provide gcc-14/g++-14/cpp-14, plus the default
            // GCC for everything else that needs a plain `cc`
            "gcc",
            "gcc-c++",
            "gcc14",
            "gcc14-c++",
        ],
    )
}

fn install_arch_command() -> Command {
    privileged_command(
        "pacman",
        &[
            "-Syu",
            // The counterpart to apt/dnf's `-y`: without it pacman asks `[Y/n]` on stdout, which
            // `Context::run` sends to the log file where nobody can see it.
            "--noconfirm",
            // Don't reinstall what's already there, so re-running `setup` is cheap.
            "--needed",
            "autoconf",
            "automake",
            "libtool",
            "pkgconf",
            "clang",
            "bison",
            "cmake",
            "ninja",
            "samba",
            "flex",
            "texinfo",
            "time",
            "glib2",
            "pixman",
            "libarchive",
            "bzip2",
            "attr",
            "libcap-ng",
            "inetutils",
            "mercurial",
            "expat",
            "gmp",
        ],
    )
}
