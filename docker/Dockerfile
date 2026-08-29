# CI image with the full CHERI Morello Rust toolchain
#
# The builder stage runs the whole crability pipeline (setup -> fetch -> llvm
# -> rust), which compiles Morello LLVM and the Rust fork from source
#
# Build args:
#   WITH_QEMU=1   also build CHERI QEMU via cheribuild and install
#                 qemu-system-morello into the image
FROM ubuntu:24.04 AS base

ENV DEBIAN_FRONTEND=noninteractive

# `crability llvm` clones Morello LLVM with `git clone --revision`, which
# needs git >= 2.49 - newer than what 24.04 ships, hence the git-core PPA.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates curl git sudo python3 build-essential tzdata \
        software-properties-common \
    && add-apt-repository -y ppa:git-core/ppa \
    && apt-get update \
    && apt-get install -y git \
    && rm -rf /var/lib/apt/lists/*

# cheribuild complains when run as root, so the toolchain is built (and later
# used) as `morello`. Passwordless sudo lets `crability setup` install the OS
# packages itself.
RUN useradd --create-home --shell /bin/bash morello \
    && echo 'morello ALL=(ALL) NOPASSWD:ALL' > /etc/sudoers.d/morello

USER morello
WORKDIR /home/morello
ENV PATH=/home/morello/.cargo/bin:$PATH

# Host Rust toolchain, only used to build crability itself
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain stable

# Copy only what `cargo install` needs: anything else from the repo (README,
# the Dockerfile itself, ...) would invalidate this layer on every edit and
# cascade into rebuilding the whole toolchain.
COPY --chown=morello:morello Cargo.toml Cargo.lock /home/morello/crability-src/
COPY --chown=morello:morello src /home/morello/crability-src/src
RUN cargo install --path /home/morello/crability-src \
    && rm -rf ~/.cargo/registry ~/.cargo/git

FROM base AS builder

# The toolchain build. One layer per step so an interrupted build resumes
# from the last completed stage instead of starting over.
RUN crability init
RUN sudo apt-get update \
    && crability setup \
    && sudo rm -rf /var/lib/apt/lists/*
RUN crability fetch
# Both steps resume incrementally, so retries absorb a transient failure
# instead of discarding the whole layer. crability pins the cheribuild host
# compiler to the gcc-14 that `crability setup` installed.
RUN crability llvm || crability llvm || crability llvm
RUN crability rust || crability rust || crability rust

ARG WITH_QEMU=0
# CHERI QEMU's meson build still imports distutils, which Python 3.12
# removed; python3-setuptools provides a compatible shim.
RUN if [ "$WITH_QEMU" = "1" ]; then \
        sudo apt-get update \
        && sudo apt-get install -y --no-install-recommends python3-setuptools \
        && sudo rm -rf /var/lib/apt/lists/* \
        && python3 ~/.crability/repos/cheribuild/cheribuild.py --skip-update -d -f qemu \
            --clang-path=/usr/bin/gcc-14 --clang++-path=/usr/bin/g++-14 \
            --clang-cpp-path=/usr/bin/cpp-14 \
        && crability qemu-install; \
    fi

# Prune what the runtime image doesn't need. The symlinks in ~/.crability/bin
# point into repos/rust/build/*/stage{0,1} and ~/cheri/output; those stay.
RUN rm -rf \
        ~/cheri/morello-llvm-project \
        ~/cheri/build \
        ~/.crability/repos/rust/.git \
        ~/.crability/repos/cheribuild/.git \
        ~/.crability/repos/rust/build/bootstrap \
        ~/.crability/repos/rust/build/cache \
        ~/.crability/repos/rust/build/tmp \
        ~/.crability/repos/rust/build/x86_64-unknown-linux-gnu/stage0-* \
        ~/.crability/repos/rust/build/x86_64-unknown-linux-gnu/stage1-* \
        ~/.local/state/crability \
        ~/.rustup

FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

# git + build-essential so checkouts and build scripts / proc-macros (which
# compile for the host) work inside CI jobs. The lib* packages are
# qemu-system-morello's runtime dependencies (tiny, so installed
# unconditionally rather than plumbing WITH_QEMU through).
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates curl git build-essential python3 \
        libglib2.0-0t64 libpixman-1-0 libcap-ng0 libbz2-1.0 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /bin/bash morello

COPY --from=builder --chown=morello:morello /home/morello/.config /home/morello/.config
COPY --from=builder --chown=morello:morello /home/morello/.crability /home/morello/.crability
COPY --from=builder --chown=morello:morello /home/morello/cheri/output /home/morello/cheri/output
COPY --from=builder /home/morello/.cargo/bin/crability /usr/local/bin/crability

# crability resolves its config relative to $HOME, so it must be
# /home/morello. GitHub Actions `container:` jobs override HOME to
# /github/home -- set `HOME: /home/morello` in the job env (see README).
ENV HOME=/home/morello \
    PATH=/home/morello/.crability/bin:$PATH \
    CARGO_TARGET_AARCH64_UNKNOWN_NONE_PURECAP_LINKER=/home/morello/.crability/repos/rust/clang-morello.sh

# Links the GHCR package to its source repository
LABEL org.opencontainers.image.source=https://github.com/sabikura/crability \
    org.opencontainers.image.description="CHERI Morello Rust toolchain built by crability"

WORKDIR /home/morello
CMD ["/bin/bash"]
