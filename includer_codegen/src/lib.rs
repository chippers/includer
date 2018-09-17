#[macro_use]
extern crate quote;
#[cfg(feature = "web")]
extern crate mime;
#[cfg(feature = "web")]
extern crate mime_guess;
extern crate proc_macro2;
pub extern crate regex;
extern crate walkdir;

mod assets;
pub mod prelude;
pub mod utils;

pub use assets::*;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

const DEFAULT_FILENAME: &str = "assets.rs";

/// A code generation builder.
///
/// By default the path written to is `$OUT_DIR/assets.rs` where `$OUT_DIR` is
/// from your environmental variables.  This is usually set by cargo.  If
/// `$OUT_DIR` is not set, then no path is set.
///
/// An unlimited amount of [`Pipelines`] can be added and they will be written
/// to the file in the same order as they were added.
///
/// [`Pipelines`]: ./trait.Pipeline.html
#[derive(Default)]
pub struct Codegen {
    assets_builder: Vec<Box<Pipeline>>,
    path: Option<PathBuf>,
}

impl Codegen {
    /// Creates a Codegen instance.
    ///
    /// If the cargo env var `OUT_DIR` is set, the path is automatically set
    /// to `$OUT_DIR/assets.rs`, otherwise a path is **not** set.  In that case
    /// you are able to explicitly able to specify the path with [`set_path`].
    ///
    /// ```
    /// use includer_codegen::prelude::*;
    ///
    /// let c = Codegen::new();
    /// ```
    ///
    /// [`set_path`]: #method.set_path
    pub fn new() -> Codegen {
        Codegen {
            assets_builder: Vec::new(),
            path: env::var("OUT_DIR")
                .ok()
                .map(PathBuf::from)
                .map(|dir| dir.join(DEFAULT_FILENAME)),
        }
    }

    /// Returns the currently set path.
    ///
    /// ```
    /// use std::path::Path;
    /// use includer_codegen::prelude::*;
    ///
    /// let c = Codegen::new().set_path("./out/gen.rs");
    /// assert_eq!(c.path(), Some(Path::new("./out/gen.rs")));
    /// ```
    pub fn path(&self) -> Option<&Path> {
        self.path.as_ref().map(PathBuf::as_path)
    }

    /// Sets the output path for the generated file.
    ///
    /// ```
    /// use includer_codegen::prelude::*;
    ///
    /// Codegen::new().set_path("./out/gen.rs");
    /// ```
    pub fn set_path<P: Into<PathBuf>>(mut self, path: P) -> Codegen {
        self.path = Some(path.into());
        self
    }

    /// Returns a list of all currently set Pipelines
    pub fn pipelines(&self) -> &[Box<Pipeline>] {
        &self.assets_builder
    }

    /// Add a [`Pipeline`] to the `Codegen` instance.
    ///
    /// ```
    /// use includer_codegen::prelude::*;
    ///
    /// Codegen::new().pipe(Assets::new("ASSETS", "../web/dist").build());
    /// ```
    ///
    /// [`Pipeline`]: ./trait.Pipeline.html
    pub fn pipe(mut self, generator: Box<Pipeline>) -> Codegen {
        self.assets_builder.push(generator);
        self
    }

    /// Writes everything to file and returns the written amount.
    ///
    /// ```no_run
    /// use includer_codegen::prelude::*;
    ///
    /// Codegen::new()
    ///     .pipe(Assets::new("ASSETS", "../web/dist").build())
    ///     .write();
    /// ```
    ///
    /// # Panics
    ///
    /// If `path` is not set then this function will panic.
    ///
    /// A panic will also happen when any file operation fails - such as
    /// opening, writing, or closing.
    pub fn write(&self) -> usize {
        let path = self.path.as_ref().expect("Codegen output path not set");
        let file = File::create(path).expect("Unable to open a file at the path");
        let mut writer = BufWriter::new(file);
        let mut written = 0;

        for assets in &self.assets_builder {
            written += writer
                .write(assets.to_string().as_bytes())
                .expect("Unable to write to Codegen file");
        }

        writer
            .flush()
            .expect("Unable to close written Codegen file");
        println!("written {} bytes to {}", written, path.to_str().unwrap());
        written
    }
}

/// Assets Pipeline.
///
/// `Pipeline` should be implemented on anything that generates code which
/// embeds assets in binaries at compile time.  For use in [`Codegen`] which
/// generates a rust file at build time with the contents of all asset
/// pipelines.
///
/// [`Codegen`]: ./struct.Codegen.html
pub trait Pipeline: ToString {
    /*/// Generates a string of Rust code to be inserted in the [`Codegen`] file.
    ///
    /// [`Codegen`]: ./struct.Codegen.html
    fn generate(&self) -> String;*/
}
