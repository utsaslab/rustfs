set environment RUST_BACKTRACE=1
set environment SPDK_INSTALL_DIR $HOME/spdk_install
set environment RUSTFLAGS "-C link-arg=$SPDK_INSTALL_DIR/lib/libspdk.so"
