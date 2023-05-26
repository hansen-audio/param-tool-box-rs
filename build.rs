// build.rs
extern crate cbindgen;

use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::generate(&crate_dir)
        .unwrap()
        .write_to_file("include/hao/param-tool-box-rs/cbindings.h");
}