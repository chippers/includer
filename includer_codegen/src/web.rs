use proc_macro2::{Ident, Span};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use utils::FilterList;
use walkdir::WalkDir;
use Pipeline;

struct AssetInfo {
    path: String,
    clean_path: String,
    has_gz: bool,
    has_br: bool,
}

enum Compressions {
    GZIP,
    BROTLI,
}

pub struct AssetsBuilder {
    ident: String,
    prefix: String,
    filter_type: FilterList,
    include: Vec<String>,
    exclude: Vec<String>,
    brotli: bool,
    gzip: bool,
    path: PathBuf,
}

impl AssetsBuilder {
    /// Creates a new generator
    ///
    /// By default the filter type is a blacklist.
    pub fn new<S: Into<String>, P: Into<PathBuf>>(identifier: S, path: P) -> Self {
        AssetsBuilder {
            ident: identifier.into(),
            filter_type: FilterList::Blacklist,
            include: Vec::new(),
            exclude: Vec::new(),
            prefix: "/".to_string(),
            brotli: true,
            gzip: true,
            path: path.into(),
        }
    }

    /// Add a file extension to be included in the whitelist
    ///
    /// If you wanted to include all javascript and css files in one grouping, you could do
    ///
    /// ```
    /// # use includer_codegen::AssetsBuilder;
    /// # use std::path::PathBuf;
    /// AssetsBuilder::new("ASSETS", PathBuf::from("./web/dist"))
    ///     .whitelist()
    ///     .include("js")
    ///     .include("css");
    /// ```
    ///
    /// If the filter is a whitelist and there are no include rules,
    /// then no files are accepted.
    ///
    /// NOTE: When gzip and/or brotli is enabled, files ending with `.gz` or `.br`
    /// (respectively) will always be ignored.  They will be embedded too, along side
    /// the uncompressed asset.
    pub fn include<S: Into<String>>(mut self, ext: S) -> Self {
        self.include.push(ext.into());
        self
    }

    /// Add a file extension to be included to the blacklist
    ///
    /// If the filter type is a blacklist and there are no exclude rules,
    /// then all files are accepted.
    ///
    /// NOTE: When gzip and/or brotli is enabled, files ending with `.gz` or `.br`
    /// (respectively) will always be ignored.  They will be embedded too, along side
    /// the uncompressed asset.
    pub fn exclude<S: Into<String>>(mut self, ext: S) -> Self {
        self.exclude.push(ext.into());
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

    /// Sets filtering to a blacklist, using `exclude` regex's
    pub fn blacklist(mut self) -> Self {
        self.filter_type = FilterList::Blacklist;
        self
    }

    /// Sets filtering to a whitelist, using `include` regex's
    pub fn whitelist(mut self) -> Self {
        self.filter_type = FilterList::Whitelist;
        self
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

    pub fn set_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.path = path.into();
        self
    }

    pub fn build(self) -> Box<Self> {
        Box::new(self)
    }
}

impl Pipeline for AssetsBuilder {
    // TODO: write this function
    fn generate(&self) -> String {
        let mut entries = Vec::new();
        for maybe_entry in WalkDir::new(&self.path) {
            let entry = maybe_entry.unwrap();

            // Make build.rs rerun if any of the dirs or files walked over are changed
            println!("cargo:rerun-if-changed={}", entry.path().to_str().unwrap());

            // We don't have special rules for directories, but we can't use walkdir's filter
            // because we don't want directories to be skipped
            if entry.file_type().is_dir() {
                continue;
            }

            println!("{}", entry.path().to_str().unwrap());

            match self.filter_type {
                FilterList::Blacklist => {
                    let ext = entry.path().extension();
                    for exclude in &self.exclude {
                        if ext == Some(exclude.as_ref()) {
                            continue;
                        }
                    }

                    if !skip_compressed(&self, ext) {
                        entries.push(PathBuf::from(entry.path()));
                    }
                }
                FilterList::Whitelist => {
                    let ext = entry.path().extension();
                    for include in &self.include {
                        if ext == Some(include.as_ref()) && !skip_compressed(&self, ext) {
                            entries.push(PathBuf::from(entry.path()))
                        }
                    }
                }
            }
        }

        println!("{:?}", entries);

        let asset_info: Vec<AssetInfo> = entries
            .iter()
            .map(|p| AssetInfo {
                path: p.to_str().unwrap().to_string(),
                clean_path: normalize_path(p, &self.path, &self.prefix),
                has_gz: compressed_exists(&p, Compressions::GZIP),
                has_br: compressed_exists(&p, Compressions::BROTLI),
            })
            .collect();

        generate_asset_const(&self.ident, asset_info)
    }
}

fn compressed_exists(path: &Path, compression: Compressions) -> bool {
    let ext = match compression {
        Compressions::GZIP => ".gz",
        Compressions::BROTLI => ".br",
    };

    let f = path.file_name().unwrap();
    let new_f = format!("{}{}", f.to_str().unwrap(), ext);
    let mut p = PathBuf::from(path);
    p.set_file_name(new_f);
    Path::exists(&p)
}

fn skip_compressed(builder: &AssetsBuilder, ext: Option<&OsStr>) -> bool {
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
