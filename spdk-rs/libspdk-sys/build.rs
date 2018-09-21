/*************************************************************************
  > File Name:       build.rs
  > Author:          Zeyuan Hu
  > Mail:            iamzeyuanhu@utexas.edu
  > Created Time:    9/16/18
  > Description:
    
    The build script for the package.
 ************************************************************************/


extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    // hzy: Don't use SPDK_DIR as environment variable here as SPDK 18.07 rely on this variable to
    // build (i.e. will fail the SPDK build if we use the same environment variable here)
    let _spdk_install_dir = match env::var("SPDK_INSTALL_DIR") {
        Ok(val) => val,
        Err(_e) => panic!("SPDK_INSTALL_DIR is not defined in the environment")
    };

    let include_path_spdk_dir = format!("-I{}/include", _spdk_install_dir);

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        .clang_arg(include_path_spdk_dir)
        // The input header we would like to generate
        // bindings for.
        .header("src/wrapper.h")
        .blacklist_type("IPPORT_.*")   // https://github.com/rust-lang-nursery/rust-bindgen/issues/687
        .blacklist_type("max_align_t") // https://github.com/rust-lang-nursery/rust-bindgen/issues/550
        .opaque_type("spdk_nvme_feat_async_event_configuration") // https://github.com/rust-lang-nursery/rust-bindgen/issues/687
        .opaque_type("spdk_nvme_feat_async_event_configuration__bindgen_ty_1")
        .rustfmt_bindings(true)
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=./build.rs");
    println!("cargo:rerun-if-changed=src/wrapper.h");
}