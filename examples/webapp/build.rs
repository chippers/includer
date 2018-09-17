extern crate includer_codegen;

use includer_codegen::prelude::*;
use std::env::var;
use std::path::PathBuf;

fn main() {
    let cargo_dir = var("CARGO_MANIFEST_DIR").unwrap();
    let dist = PathBuf::from(cargo_dir).join("web/dist");

    let webpack = WebAssets::new("ASSETS", dist).build();

    Codegen::new().pipe(webpack).write();
}
