extern crate includer;
#[cfg(test)]
extern crate select;

use includer::WebAsset;

include!(concat!(env!("OUT_DIR"), "/assets.rs"));

/// Prints the debug version of your assets (note: bytes not strings)
fn main() {
    println!("{:?}", ASSETS);
}

/// Returns the bytes of index.html from your included assets
///
/// friendly reminder: usually you wouldn't be searching for your manifest
/// (thinking of index.html as a manifest of your webapp assets) in your code,
/// ideally you should know the manifest beforehand and even possibly keep it
/// separate from your other included assets.
pub fn index() -> &'static [u8] {
    let idx = find_index_index(&ASSETS); // see function comment
    ASSETS[idx].data()
}

/// Returns the index of index.html from the raw generated `WebAsset` array
///
/// yeah confusing and not the best way but i found the name funny
fn find_index_index(assets: &[WebAsset]) -> usize {
    assets.iter()
        .position(|a| a.uri() == "/") // normalized to just "/" because its a web asset

        // reminder: don't do this in production, just simple for this test
        .expect("Not able to find index.html, this shouldn't happen")
}

#[cfg(test)]
mod tests {
    use select::document::Document;
    use select::node::Node;
    use select::predicate::Name;
    use std::str;

    #[test]
    fn index_exists() {
        let index = super::index();
        assert!(index.len() > 0)
    }

    /// friendly reminder: this isn't a good test for an actual production
    /// environment, its a very limited scoped test that assumes that there
    /// is exactly 1 <script> html tag in `/index.html`.  Testing for amount of
    /// external scripts/styles can have its place in a real codebase, but this
    /// is a very narrow test of exactly what the example webapp is supposed to
    /// show. if you know exactly what this entails and you understand why this
    /// might be an applicable test for your codebase, ignore this but it
    /// should be a well understood problem.  please open an issue if you may
    /// have a better way of showing this.
    #[test]
    fn index_requires_included_assets() {
        let index = super::index();
        let index_s = str::from_utf8(&index).unwrap();
        let document = Document::from(index_s);

        // note that this isn't necessarily a good test for a production
        // codebase, this is just for this simple example test.
        let scripts: Vec<Node> = document.find(Name("script")).collect();
        assert_eq!(scripts.len(), 1);

        // testing that we have the specific js asset that is required by
        // index.html in our included assets.  reminder: yada yada yada not
        // for production unless you really know what you're doing.
        let js = scripts[0];
        let src = js.attr("src").unwrap();
        assert!(super::ASSETS.iter().any(|asset| asset.uri() == src));
    }
}
