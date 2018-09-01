extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let spdk_dir = match env::var("SPDK_DIR") {
        Ok(val) => val,
        Err(_e) => panic!("SPDK_DIR is not defined in the environment")
    };
    let include_path = format!("-I{}/include", spdk_dir);

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        .clang_arg(include_path)
        // The input header we would like to generate
        // bindings for.
        .header("src/wrapper.h")
        .blacklist_type("IPPORT_.*")   // https://github.com/rust-lang-nursery/rust-bindgen/issues/687
        .blacklist_type("max_align_t") // https://github.com/rust-lang-nursery/rust-bindgen/issues/550
        .opaque_type("spdk_nvme_feat_async_event_configuration") // https://github.com/rust-lang-nursery/rust-bindgen/issues/687
        .opaque_type("spdk_nvme_feat_async_event_configuration__bindgen_ty_1")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}