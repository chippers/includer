//#[cfg(feature = "web")]
//mod web;

use proc_macro2::{Ident, Span};
use std::path::{Path, PathBuf};
use utils;
use utils::Filter;
use utils::Filter::*;
use utils::FilterListType;
use utils::FilterListType::*;
use utils::FilterRule;
use walkdir::WalkDir;
use Pipeline;

//#[cfg(feature = "web")]
//pub use self::web::*;

struct AssetInfo {
    path: String,
    clean_path: String,
}

pub struct Assets {
    ident: String,
    prefix: String,
    filters: Vec<Filter>,
    filter_list_type: FilterListType,
    path: PathBuf,
}

impl Assets {
    /// Creates a new `Assets` Pipeline
    ///
    /// By default, the filter list type is a blacklist.
    pub fn new<S: Into<String>, P: Into<PathBuf>>(identifier: S, path: P) -> Self {
        Assets {
            ident: identifier.into(),
            filters: Vec::new(),
            filter_list_type: Blacklist,
            prefix: "/".to_string(),
            path: path.into(),
        }
    }

    /// Add a filter to the pipeline
    ///
    /// Filters are applied in the order that they were added, the first
    /// matching filter determines how the file entry is handled.  If you want
    /// to include all Lua files, but not the ones in a certain folder, then
    /// you should add the exclusion rule first, and then the inclusion filter.
    ///
    /// If there are no filters then all files are matched.  If there are no
    /// filters and it's a whitelist, no files are matched.
    ///
    /// # Examples
    ///
    /// Matching everything except png and jpg files:
    ///
    /// ```
    /// # use includer_codegen::prelude::*;
    /// #
    /// Assets::new("NON_IMAGE_ASSETS", "../resources")
    ///     .filter(Filter::exclude_extension("png"))
    ///     .filter(Filter::exclude_extension("jpg"))
    ///     .filter(Filter::exclude_extension("jpeg"));
    /// ```
    ///
    /// Include all Lua files and exclude all files in the `admin` subdirectory:
    ///
    /// ```
    /// # use includer_codegen::prelude::*;
    /// #
    /// Assets::new("ASSETS", "../lua_src")
    ///     .whitelist()
    ///     .filter(Filter::exclude_regex(r"^admin/.*$"))
    ///     .filter(Filter::include_extension("lua"));
    /// ```
    ///
    /// If you wanted to only match text and markdown files:
    ///
    /// ```
    /// # use includer_codegen::prelude::*;
    /// #
    /// Assets::new("ASSETS", "../notes")
    ///     .whitelist()
    ///     .filter(Filter::include_extension("txt"))
    ///     .filter(Filter::include_extension("md"));
    /// ```
    ///
    /// or to include all assets in a subdirectory `styles`:
    ///
    /// ```
    /// # use includer_codegen::prelude::*;
    /// #
    /// Assets::new("ASSETS", "../web/dist")
    ///     .whitelist()
    ///     .filter(Filter::include_regex(r"^styles/.*$"));
    /// ```
    ///
    /// NOTE: If you need to care about multi-platform it's your responsibility
    /// to use a proper regex that accounts for the proper path separator.
    /// See [`FilterRule::regex`] for examples that account for this.
    /// AFAIK `std::path` doesn't normalize the separators.
    ///
    /// [`FilterRule::regex`]: ./utils/enum.FilterRule.html#method.regex
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Sets the prefix to use for the normalized path uri.
    ///
    /// This is relative to where your asset path is.  If your asset path is `"./web/dist/assets"`,
    /// with your web root being at `"./web/dist"`, then having a prefix of `"/assets"` would make
    /// the relative URIs align with your web root to make the hit URL correct.
    ///
    /// Defaults to `"/"`
    pub fn prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Sets the path to the assets directory.
    pub fn set_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.path = path.into();
        self
    }

    /// Set the filter list type to a blacklist.
    pub fn blacklist(mut self) -> Self {
        self.filter_list_type = Blacklist;
        self
    }

    /// Set the filter list type to a whitelist.
    pub fn whitelist(mut self) -> Self {
        self.filter_list_type = Whitelist;
        self
    }

    /// Boxes up the pipeline to pass to [`Codegen`] easily.
    ///
    /// [`Codegen`]: ../struct.Codegen.html
    pub fn build(self) -> Box<Self> {
        Box::new(self)
    }
}

impl ToString for Assets {
    fn to_string(&self) -> String {
        let mut entries = Vec::new();
        for maybe_entry in WalkDir::new(&self.path) {
            let entry = maybe_entry.expect("Couldn't read DirEntry");

            // We don't have special rules for directories, but we can't use
            // walkdir's entry filter because we don't want files under
            // directories to be skipped.
            if entry.file_type().is_dir() {
                utils::watch_path(entry.path());
                continue;
            }

            if self.filters.is_empty() {
                match self.filter_list_type {
                    Whitelist => break,
                    Blacklist => {
                        utils::watch_path(entry.path());
                        entries.push(PathBuf::from(entry.path()));
                        continue;
                    }
                }
            }

            let mut matched = true;
            for filter in &self.filters {
                // Skip all filters that don't match the entry
                if !filter.matches(entry.path()) {
                    continue;
                }

                if let Exclude(_) = filter {
                    matched = false;
                }

                break;
            }

            if matched {
                utils::watch_path(entry.path());
                entries.push(PathBuf::from(entry.path()));
            }
        }

        let asset_info: Vec<AssetInfo> = entries
            .iter()
            .map(|p| AssetInfo {
                path: utils::path_to_string(p),
                clean_path: normalize_path(p, &self.path, &self.prefix),
            }).collect();

        if asset_info.is_empty() {
            panic!("No assets were matched, something is wrong")
        }

        generate_asset_const(&self.ident, asset_info)
    }
}

impl Pipeline for Assets {}

fn normalize_path(path: &Path, dir: &Path, prefix: &str) -> String {
    let relative = path.strip_prefix(&dir).expect("Couldn't strip path prefix");
    let path = PathBuf::from("/").join(prefix).join(&relative);
    utils::path_to_string(path)
}

fn generate_asset_const(ident_str: &str, raw_assets: Vec<AssetInfo>) -> String {
    let len = raw_assets.len();
    let mut structs = Vec::new();

    for AssetInfo { path, clean_path } in raw_assets {
        structs.push(quote! {
            Asset {
                uri: #clean_path,
                data: include_bytes!(#path),
            }
        });
    }

    let ident = Ident::new(ident_str, Span::call_site());

    let tokens = quote! {
        const #ident: [Asset; #len] = [#(#structs),*];
    };

    format!("{}", tokens)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(1 + 1, 2);
    }
}
