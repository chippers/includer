# includer_codegen [![Build Status](https://travis-ci.org/chippers/includer.svg?branch=master)](https://travis-ci.org/chippers/includer) [![Documentation](https://docs.rs/includer_codegen/badge.svg)](https://docs.rs/includer_codegen)

This crate is intended to be used at build time to generate code that includes
your assets.  The "frontend" library [`includer`] provides the types for your
library/binary that `includer_codegen` outputs, along with some helpers.

[`includer`]: https://crates.io/crates/includer

The following is a `build.rs` file that includes all the files (recursively) in
the subdirectory `resources` in a cargo project.

```rust
extern crate includer_codegen;

use includer_codegen::prelude::*;
use std::env;
use std::path::PathBuf;

fn main() {
    let cargo_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let resources_path = PathBuf::from(cargo_dir).join("resources");
    let resources = Assets::new("ASSETS", resources_path).build();

    Codegen::new().pipe(resources).write();
}

```

## Filtering

Filtering files is possible by included filter types.  Currently there are
built-in ways to include/exclude based on file extension or regex.  See the
[documentation](https://docs.rs/includer_codegen) for the api to use these
built-in filters.


## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
