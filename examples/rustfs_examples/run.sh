#!/bin/bash

# All commands should be run with `sudo`

source $HOME/.cargo/env
export RUST_BACKTRACE=1

# Pointing `SPDK_INSTALL_DIR` to the installation location of SPDK and run the following commands:
export SPDK_INSTALL_DIR=$HOME/spdk_install
export RUSTFLAGS="-C link-arg=$SPDK_INSTALL_DIR/lib/libspdk.so"

# Logging level
export RUSTFS_BENCHMARKS_LANGUAGE_LOG=debug

# Whether to use memory (e.g., Malloc0) instead of NVMe disk for the benchmark
export MALLOC0=0

cargo run
