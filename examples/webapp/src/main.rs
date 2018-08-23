extern crate includer;

use includer::Asset;

include!(concat!(env!("OUT_DIR"), "/assets.rs"));

fn main() {
    println!("{:?}", ASSETS);
}
