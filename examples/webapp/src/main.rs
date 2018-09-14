extern crate includer;
#[cfg(test)]
extern crate select;

use includer::Asset;

include!(concat!(env!("OUT_DIR"), "/assets.rs"));

/// Prints the debug version of your assets (note: bytes not strings)
fn main() {
    println!("{:?}", ASSETS);
}

/// Returns the bytes of index.html from your included assets
///
/// friendly reminder: usually you wouldn't be searching for your manifest
/// (thinking of index.html as a manifest of your webapp assets) in your code,
/// ideally you should know the manifest beforehand and possibly keep it
/// separate from your other included assets.
pub fn index() -> &'static [u8] {
    let idx = find_index_index(&ASSETS); // see function comment
    ASSETS[idx].data()
}

/// Returns the index of your index.html from the raw generated `Asset` array
///
/// yeah confusing and not the best way but i found the name funny
fn find_index_index(assets: &[Asset]) -> usize {
    assets.iter()
        .position(|a| a.uri() == "/index.html")

        // reminder: don't do this in production, just simple for this test
        .expect("Not able to find index.html, this shouldn't happen")
}

#[cfg(test)]
mod tests {
    use super::{index, ASSETS};
    use select::document::Document;
    use select::node::Node;
    use select::predicate::Name;
    use std::str;

    #[test]
    fn index_exists() {
        let index_bytes = index();
        assert!(index_bytes.len() > 0)
    }

    // friendly reminder: this isn't a good test for an actual production
    // environment, its a very limited scoped test that assumes that there
    // is exactly 1 <script> html tag in `/index.html`.  Testing for amount of
    // external scripts/styles can have its place in a real codebase, but this
    // is a very narrow test of exactly what the example webapp is supposed to
    // show. if you know exactly what this entails and you understand why this
    // might be an applicable test for your codebase, ignore this but it should
    // be a well understood problem.  please open an issue if you may have a
    // better way of showing this.
    #[test]
    fn index_requires_included_assets() {
        let actual_index = index();
        let index_str = str::from_utf8(&actual_index).unwrap();
        let document = Document::from(index_str);

        // note that this isn't necessarily a good test for a production
        // codebase, this is just for this simple example test.
        let scripts: Vec<Node> = document.find(Name("script")).collect();
        assert_eq!(scripts.len(), 1);

        // testing that we have the specific js asset that is required by
        // index.html in our included assets.  reminder: yada yada yada not
        // for production unless you really know what you're doing.
        let js_script = scripts[0];
        let src_str = js_script.attr("src").unwrap();
        assert!(ASSETS.iter().any(|asset| asset.uri() == src_str));
    }
}
