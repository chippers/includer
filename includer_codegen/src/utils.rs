//! Tools to build custom `Pipeline`s that aren't already provided.
//!
//! Using the tools here to build custom [`Pipelines`] can help keep your
//! pipelines more consistent, and easier to use for others.
//!
//! [`Pipelines`]: ../trait.Pipeline.html
use self::Filter::*;
use regex::Regex;
use std::path::Path;

/// The type of filter list to use.
pub enum FilterListType {
    Blacklist,
    Whitelist,
}

/// A filter to determine if a file or path should be added to the `Pipeline`.
pub enum Filter {
    Include(FilterRule),
    Exclude(FilterRule),
}

impl Filter {
    /// Create a filter that includes a file extension.
    ///
    /// Can panic, see [`FilterRule::extension`].
    ///
    /// [`FilterRule::extension`]: ./enum.FilterRule.html#method.extension
    pub fn include_extension<S: Into<String>>(ext: S) -> Self {
        Include(FilterRule::extension(ext))
    }

    /// Create a filter that excludes a file extension.
    ///
    /// Can panic, see [`FilterRule::extension`].
    ///
    /// [`FilterRule::extension`]: ./enum.FilterRule.html#method.extension
    pub fn exclude_extension<S: Into<String>>(ext: S) -> Self {
        Exclude(FilterRule::extension(ext))
    }

    /// Create a filter that includes a regex.
    ///
    /// Can panic, see [`FilterRule::regex`].
    ///
    /// [`FilterRule::regex`]: ./enum.FilterRule.html#method.regex
    pub fn include_regex<S: AsRef<str>>(regex_str: S) -> Self {
        Include(FilterRule::regex(regex_str))
    }

    /// Create a filter that excludes a regex.
    ///
    /// Can panic, see [`FilterRule::regex`].
    ///
    /// [`FilterRule::regex`]: ./enum.FilterRule.html#method.regex
    pub fn exclude_regex<S: AsRef<str>>(regex_str: S) -> Self {
        Exclude(FilterRule::regex(regex_str))
    }

    pub fn matches<P: AsRef<Path>>(&self, relative_path: P) -> bool {
        match self {
            Include(rule) => rule.matches(relative_path),
            Exclude(rule) => rule.matches(relative_path),
        }
    }
}

/// A rule on how to match a file or path
pub enum FilterRule {
    /// Match any file that contains the specified extension.
    ///
    /// It is suggested to use the [`extension`] helper method instead for
    /// ease of use and consistency.
    ///
    /// ```
    /// # use includer_codegen::utils::FilterRule;
    /// #
    /// let html_files = FilterRule::Extension("html".to_string());
    /// let inconsistent = FilterRule::Extension(".html".to_string());
    /// ```
    ///
    /// Note: For consistency, the extension should *not* have a leading period
    ///
    /// [`extension`]: #method.extension
    Extension(String),

    /// Match any file that has a regex match on its path.
    ///
    /// It is suggested to use the [`regex`] helper method instead for
    /// ease of use and consistency.
    ///
    /// ```
    /// # use includer_codegen::utils::FilterRule;
    /// use includer_codegen::regex::Regex;
    ///
    /// // Match all css files in the "styles" subdirectory
    /// let css_files = FilterRule::Regex(Regex::new(r"^styles[/\\].*\.css$").unwrap());
    /// ```
    ///
    /// Note: For consistency, the path compared to the regex should be
    /// relative to the root asset path with no leading slash.  You can get
    /// this result by using [`strip_prefix`].
    ///
    /// [`regex`]: #method.regex
    /// [`strip_prefix`]: https://doc.rust-lang.org/std/path/struct.Path.html#method.strip_prefix
    Regex(Regex),
}

impl FilterRule {
    /// Creates a validated `FilterRule::Extension`
    ///
    /// Makes sure that the extension does not have a leading `"."`.
    ///
    /// ```
    /// # use includer_codegen::utils::FilterRule;
    /// #
    /// // Only accept HTML files
    /// FilterRule::extension("html");
    /// ```
    ///
    /// # Panics
    ///
    /// This function will panic if extension is not valid.
    ///
    /// ```should_panic
    /// # use includer_codegen::utils::FilterRule;
    /// #
    /// // should panic
    /// FilterRule::extension(".html");
    /// ```
    pub fn extension<S: Into<String>>(extension: S) -> Self {
        let ext = extension.into();

        if &ext[0..1] == "." {
            panic!("Filter::Extension should not contain a period prefix!");
        }

        FilterRule::Extension(ext)
    }

    /// Creates a validated `Filer::Regex`
    ///
    /// ```
    /// # use includer_codegen::utils::FilterRule;
    /// #
    /// // Accept all css files that are under the root subdirectory `styles` (multi-platform)
    /// FilterRule::regex(r"^styles[/\\].*\.css$");
    /// ```
    ///
    /// # Panics
    ///
    /// Invalid regex expressions will panic.
    ///
    /// ```should_panic
    /// # use includer_codegen::utils::FilterRule;
    /// #
    /// // should panic
    /// FilterRule::regex(r"\h");
    /// ```
    pub fn regex<S: AsRef<str>>(regex_str: S) -> Self {
        let regex = Regex::new(regex_str.as_ref());
        FilterRule::Regex(regex.unwrap())
    }

    /// See if the path matches the filter rule
    pub fn matches<P: AsRef<Path>>(&self, relative_path: P) -> bool {
        let path = relative_path.as_ref();
        match self {
            FilterRule::Extension(ext) => path.extension() == Some(ext.as_ref()),
            FilterRule::Regex(re) => re.is_match(
                path.to_str()
                    .expect("Path couldn't be represented by a str"),
            ),
        }
    }
}

pub(crate) fn path_to_string<P: AsRef<Path>>(path: P) -> String {
    path.as_ref()
        .to_str()
        .expect("Unable to represent path as str")
        .to_string()
}

/// Makes Cargo re-run build script if path has changed since last build.
///
/// Note this only has an effect inside a build script, as it just prints a
/// Cargo interpreted key to stdout.  See [`reference`].
///
/// # Panics
///
/// Panics if the path cannot be casted to a str.
///
/// [`reference`]: https://doc.rust-lang.org/cargo/reference/build-scripts.html#outputs-of-the-build-script
pub fn watch_path<P: AsRef<Path>>(p: P) {
    println!("cargo:rerun-if-changed={}", p.as_ref().to_str().unwrap());
}
