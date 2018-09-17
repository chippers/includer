use proc_macro2::{Ident, Span};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use utils;
use utils::Filter;
use utils::Filter::*;
use utils::FilterListType;
use utils::FilterListType::*;
use walkdir::WalkDir;
use Pipeline;

struct AssetInfo {
    path: String,
    clean_path: String,
    has_gz: bool,
    has_br: bool,
}

enum CompressionType {
    GZIP,
    BROTLI,
}

pub struct WebAssets {
    ident: String,
    prefix: String,
    path: PathBuf,
    filters: Vec<Filter>,
    filter_list_type: FilterListType,
    brotli: bool,
    gzip: bool,
}

impl WebAssets {
    /// Creates a new `WebAssets` Pipeline
    ///
    /// By default the filter list type is a blacklist.
    pub fn new<S: Into<String>, P: Into<PathBuf>>(identifier: S, path: P) -> Self {
        WebAssets {
            ident: identifier.into(),
            prefix: "/".to_string(),
            path: path.into(),
            filters: Vec::new(),
            filter_list_type: Blacklist,
            brotli: true,
            gzip: true,
        }
    }

    /// Add a filter to the pipeline
    ///
    /// Filters are applied in the order that they were added, the first
    /// matching filter determines how the file entry is handled.  If you want
    /// to include all javascript files, but not the ones in a certain folder,
    /// then you should add the exclusion rule first, and then the inclusion
    /// filter.
    ///
    /// If there are no filters then all files are matched.  If there are no
    /// filters and it's a whitelist, no files are matched.
    ///
    /// # Examples
    ///
    /// Matching everything except css and json files:
    ///
    /// ```
    /// # use includer_codegen::prelude::*;
    /// #
    /// WebAssets::new("NON_IMAGE_ASSETS", "../resources")
    ///     .filter(Filter::exclude_extension("css"))
    ///     .filter(Filter::exclude_extension("json"));
    /// ```
    ///
    /// Include all javascript files, but exclude all in the `admin`subdirectory:
    ///
    /// ```
    /// # use includer_codegen::prelude::*;
    /// #
    /// WebAssets::new("ASSETS", "./web/dist")
    ///     .whitelist()
    ///     .filter(Filter::exclude_regex(r"^admin/.*$"))
    ///     .filter(Filter::include_extension("js"));
    /// ```
    ///
    /// If you wanted to only match text and markdown files:
    ///
    /// ```
    /// # use includer_codegen::prelude::*;
    /// #
    /// WebAssets::new("ASSETS", "../notes")
    ///     .whitelist()
    ///     .filter(Filter::include_extension("txt"))
    ///     .filter(Filter::include_extension("md"));
    /// ```
    ///
    /// or to include all css files in a subdirectory `styles`:
    ///
    /// ```
    /// # use includer_codegen::prelude::*;
    /// #
    /// WebAssets::new("ASSETS", "../web/dist")
    ///     .whitelist()
    ///     .filter(Filter::include_regex(r"^styles/.*\.css$"));
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

    /// Sets whether to include the brotli version of every file too
    pub fn brotli(mut self, brotli: bool) -> Self {
        self.brotli = brotli;
        self
    }

    /// Sets whether to include the gzip version of every file too
    pub fn gzip(mut self, gzip: bool) -> Self {
        self.gzip = gzip;
        self
    }
}

impl ToString for WebAssets {
    fn to_string(&self) -> String {
        let mut entries = Vec::new();
        for maybe_entry in WalkDir::new(&self.path) {
            let entry = maybe_entry.unwrap();

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
                        if !skip_compressed(&self, entry.path().extension()) {
                            utils::watch_path(entry.path());
                            entries.push(PathBuf::from(entry.path()));
                        }
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

                if skip_compressed(&self, entry.path().extension()) {
                    matched = false;
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
                has_gz: compressed_exists(&p, CompressionType::GZIP),
                has_br: compressed_exists(&p, CompressionType::BROTLI),
            }).collect();

        generate_asset_const(&self.ident, asset_info)
    }
}

impl Pipeline for WebAssets {}

fn compressed_exists(path: &Path, compression: CompressionType) -> bool {
    let ext = match compression {
        CompressionType::GZIP => ".gz",
        CompressionType::BROTLI => ".br",
    };

    let f = path.file_name().unwrap();
    let new_f = format!("{}{}", f.to_str().unwrap(), ext);
    let mut p = PathBuf::from(path);
    p.set_file_name(new_f);
    Path::exists(&p)
}

fn skip_compressed(builder: &WebAssets, ext: Option<&OsStr>) -> bool {
    if builder.gzip && ext == Some("gz".as_ref()) {
        return true;
    }

    if builder.brotli && ext == Some("br".as_ref()) {
        return true;
    }

    false
}

fn normalize_path(path: &Path, dir: &Path, prefix: &str) -> String {
    let path = path.strip_prefix(&dir).unwrap();
    let path = PathBuf::from("/").join(prefix).join(&path);

    if path.file_name().unwrap() == "index.html" {
        path.parent().unwrap().to_str().unwrap().to_owned()
    } else {
        path.to_str().unwrap().to_owned()
    }
}

fn generate_asset_const(ident_str: &str, raw_assets: Vec<AssetInfo>) -> String {
    let len = raw_assets.len();
    let mut structs = Vec::new();

    for AssetInfo {
        path,
        clean_path,
        has_gz,
        has_br,
    } in raw_assets
    {
        let gz = if has_gz {
            let path_gz = path.clone() + ".gz";
            quote! {Some(include_bytes!(#path_gz))}
        } else {
            quote! {None}
        };

        let br = if has_br {
            let path_br = path.clone() + ".br";
            quote! {Some(include_bytes!(#path_br))}
        } else {
            quote! {None}
        };

        structs.push(quote! {
            WebAsset {
            uri: #clean_path,
            data: include_bytes!(#path),
            data_gz: #gz,
            data_br: #br,
            mime: "text/plain",
            }
        });
    }

    let ident = Ident::new(ident_str, Span::call_site());

    let tokens = quote! {
        const #ident: [WebAsset; #len] = [#(#structs),*];
    };

    format!("{}\n", tokens)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(1 + 1, 2);
    }
}
