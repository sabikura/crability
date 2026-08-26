# crability

[![CI](https://github.com/sabikura/crability/actions/workflows/ci.yml/badge.svg)](https://github.com/sabikura/crability/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Tool for setting up the environment for CHERI (currently only Aarch64 Morello) Rust development.

![demo](docs/demo.gif)

Since neither the LLVM nor the Rust support for CHERI is upstreamed yet, getting a working toolchain means juggling [cheribuild](https://github.com/CTSRD-CHERI/cheribuild), a patched [Morello LLVM](https://git.morello-project.org/morello/llvm-project) and a Rust fork with CHERI support. This CLI tool makes it easier to have this setup and gives you a `cargo` wrapper that targets Morello out of the box.

## Installation

The only option right now for installing is building from source using [cargo](https://rustup.rs/) (you will need Rust 1.85.1 or higher):

```sh
cargo install --git https://github.com/sabikura/crability.git 
```

## Getting started

`crability init` writes a JSON configuration to `~/.config/crability/config.json`. Every
repository is pinned to a specific commit that was tested to work, and everything is overridable.
Mostly the things that user may consider to configure is the path where the tool fetches
repositories (default is `~/.crability/repos`) and the path where the tool installs
the built binaries (default is `~/.crability/bin`).

Run `crability help` to find all the subcommands you can run in order to finish your setup.
After building the Rust fork successfully after the `crability rust` step, you should be able to run:

```sh
crability cargo --version
```

## Requirements

- Linux (Debian/Ubuntu, RHEL/Fedora or Arch for `crability setup`, but other distributions will work if you install cheribuild's dependencies yourself)
- `git`, and enough disk space and patience to wait for the LLVM and Rust builds to finish :>

## License

Licensed under the MIT License ([LICENSE](LICENSE) or https://opensource.org/licenses/MIT)
