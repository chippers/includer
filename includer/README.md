# includer [![Build Status](https://travis-ci.org/chippers/includer.svg?branch=master)](https://travis-ci.org/chippers/includer) [![Documentation](https://docs.rs/includer/badge.svg)](https://docs.rs/includer)

The `includer` crate is the library to be used by other libraries/binaries.
This can be considered the "frontend" counterpart to the build time crate that
helps you generate code, [`includer_codegen`].  For now it's mostly a simple
type wrapper to get the generated code that's included to compile correctly and
safely.

[`includer_codegen`]: https://crates.io/crates/includer_codegen

You would not typically use the types from this library in your codebase, but
rather only include them for the generated code to use.

```rust
extern crate includer;

use includer::Asset;

// The default file that includer_codegen generates
include!(concat!(env!("OUT_DIR"), "/assets.rs"));
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
