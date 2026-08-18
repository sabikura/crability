use anyhow::anyhow;
use anyhow::{Context, Result};
use indicatif::ProgressBar;
use nix::unistd::Uid;
use owo_colors::OwoColorize;
use std::process::Command;

/// Where we got the needed packages list from
/// TODO: Fix this to the commit in the config json
const _PRE_BUILD_SETUP_URL: &str = "https://github.com/CTSRD-CHERI/cheribuild#pre-build-setup";

/// Supported platforms for the setup step
/// TODO: Also support macOS
#[derive(Debug)]
enum Platform {
    Debian,
    Fedora,
    Arch,
}

pub(crate) fn run() -> Result<()> {
    let bar = ProgressBar::new_spinner();
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

    // NOTE: This bar is not looking that pretty...maybe just replace with a println
    bar.set_message(format!(
        "Installing {} prerequisites for the current platform: {:?}",
        "cheribuild".bold().cyan(),
        platform
    ));

    match platform {
        Platform::Debian => install_debian(),
        Platform::Fedora => install_fedora(),
        Platform::Arch => install_arch(),
    }?;

    bar.finish_with_message(format!(
        "Installed prerequisites for {}",
        "cheribuild".bold().cyan()
    ));

    Ok(())
}

fn run_privileged(program: &str, args: &[&str]) -> Result<()> {
    let mut argv = Vec::with_capacity(args.len() + 2);
    if !Uid::effective().is_root() {
        // TODO: Maybe check for doas also?
        argv.push("sudo");
    }
    argv.push(program);
    argv.extend_from_slice(args);

    // TODO: Add a `run` helper that prints the command ran
    let status = Command::new(argv[0])
        .args(&argv[1..])
        .status()
        .with_context(|| format!("Failed to run {}", argv.join(" ")))?;

    match status.code() {
        Some(0) => Ok(()),
        Some(code) => Err(anyhow!("{} exited with status {}", program, code)),
        None => Err(anyhow!("{} was terminated by a signal", program)),
    }
}

fn install_debian() -> Result<()> {
    run_privileged(
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
        ],
    )
}

fn install_fedora() -> Result<()> {
    run_privileged(
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
        ],
    )
}

fn install_arch() -> Result<()> {
    run_privileged(
        "pacman",
        &[
            "-Syu",
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
