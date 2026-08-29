use crate::{
    cheribuild_config::CheribuildPaths,
    command::install_bins_from,
    config::{self, Config, RepoConfig},
    context::Context,
    status,
};
use anyhow::{Context as _, Result, bail};
use owo_colors::OwoColorize;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

pub(crate) fn run(ctx: &mut Context) -> Result<()> {
    let paths = CheribuildPaths::read()?;
    let config = Config::load_and_print(&config::path()?)
        .context("Could not read from config file. Please run `crability init`")?;

    fs::create_dir_all(&paths.source_root)
        .with_context(|| format!("creating {}", paths.source_root.display()))?;

    let patch = config.rust_dir().join("llvm.patch");
    if !patch.exists() {
        bail!(
            "LLVM patch {} not found. Run {} first",
            patch.display().underline(),
            "crability fetch".bold()
        );
    }

    let llvm_dir = paths.morello_llvm_dir();
    let freshly_cloned = !llvm_dir.join(".git").exists();
    clone_morello_llvm(&llvm_dir, &config.morello_llvm, ctx)?;

    // Apply the patch shipped in the rust repo.
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
        ctx.run(
            Command::new("git")
                .current_dir(llvm_dir)
                .args(["apply", patch_str]),
        )?;
    }

    // Build via cheribuild. Left to its own devices cheribuild picks the
    // newest system clang for host binaries, which has crashed sporadically
    // building Morello LLVM; pin the build to GCC, the known-good host
    // compiler.
    let (cc, cxx, cpp) = find_host_gcc()?;
    status!(
        "Building with the {} host compiler",
        cc.display().underline()
    );
    ctx.run(
        Command::new("python3")
            .current_dir(config.cheribuild_dir())
            .args(["cheribuild.py", "--skip-update", "morello-llvm"])
            .arg(format!("--clang-path={}", cc.display()))
            .arg(format!("--clang++-path={}", cxx.display()))
            .arg(format!("--clang-cpp-path={}", cpp.display())),
    )
    .with_context(|| {
        format!("Failed to re-build LLVM. Please try to build with an older version of GCC.")
    })?;

    let llvm_config = paths.morello_sdk_bin().join("llvm-config");
    if !llvm_config.exists() {
        bail!(
            "cheribuild reported success but {} is missing. \
             Check cheribuild's output-root setting",
            llvm_config.display()
        );
    }

    // Install the Morello SDK binaries in ~/.crability/bin
    let bin_dir = config.bin_dir();
    status!(
        "Installing the Morello SDK binaries inside {}",
        bin_dir.display().underline()
    );
    install_bins_from(&paths.morello_sdk_bin(), &bin_dir)?;

    Ok(())
}

/// Find the GCC to hand to cheribuild, preferring the versions known to build
/// Morello LLVM. All three of gcc/g++/cpp must come from the same version, so
/// the suffixes are tried as a set (`gcc-14`/`g++-14`/`cpp-14` on
/// Debian/Ubuntu, `gcc-13`/... from Arch's gcc13, plain `gcc`/... elsewhere).
fn find_host_gcc() -> Result<(PathBuf, PathBuf, PathBuf)> {
    const SUFFIXES: &[&str] = &["-14", "-13", ""];

    for suffix in SUFFIXES {
        let found = (
            find_in_path(&format!("gcc{suffix}")),
            find_in_path(&format!("g++{suffix}")),
            find_in_path(&format!("cpp{suffix}")),
        );
        if let (Some(cc), Some(cxx), Some(cpp)) = found {
            return Ok((cc, cxx, cpp));
        }
    }

    bail!(
        "No GCC found on PATH. Run {} to install the prerequisites",
        "crability setup".bold()
    )
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}

/// Morello LLVM lives in cheribuild's source root rather than in `repos_dir`, and it is big
/// enough that we only ever clone it once, at the pinned commit.
fn clone_morello_llvm(local: &Path, config: &RepoConfig, ctx: &mut Context) -> Result<()> {
    if local.join(".git").exists() {
        let local = local.display();
        status!(
            "Morello LLVM {} already cloned, skipping",
            local.underline()
        );
        return Ok(());
    }
    if let Some(parent) = local.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    let local_str = local.to_str().context("non-UTF-8 path")?;

    ctx.run(Command::new("git").args([
        "clone",
        "--revision",
        &config.commit,
        "--depth",
        "1",
        &config.remote,
        local_str,
    ]))
}
