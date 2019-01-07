#!/bin/bash

source $HOME/.cargo/env
export RUST_BACKTRACE=1

# Pointing `SPDK_INSTALL_DIR` to the installation location of SPDK and run the following commands:
export SPDK_INSTALL_DIR=$HOME/spdk_install
export RUSTFLAGS="-C link-arg=$SPDK_INSTALL_DIR/lib/libspdk.so"

# Logging level
export RUSTFS_BENCHMARKS_LANGUAGE_LOG=debug

if [ "$1" = "run" ]; then
    cargo run
elif [ "$1" =  "clean" ]; then
    cargo clean
else
    cargo run
fi
