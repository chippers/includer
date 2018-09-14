#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "web")]
pub use web::*;

#[derive(Debug)]
pub struct Asset {
    pub uri: &'static str,
    pub data: &'static [u8],
}

impl Asset {
    pub fn uri(&self) -> &'static str {
        self.uri
    }

    pub fn data(&self) -> &'static [u8] {
        self.data
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
