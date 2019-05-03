#!/bin/bash

# All commands should be run with `sudo`

source $HOME/.cargo/env
export RUST_BACKTRACE=1

# Pointing `SPDK_INSTALL_DIR` to the installation location of SPDK and run the following commands:
export SPDK_INSTALL_DIR=$HOME/spdk_install
export RUSTFLAGS="-C link-arg=$SPDK_INSTALL_DIR/lib/libspdk.so"

cargo build
