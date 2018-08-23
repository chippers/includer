#[cfg(feature = "web")]
pub mod web;

#[derive(Debug)]
pub struct Asset {
    pub uri: &'static str,
    pub data: &'static [u8],
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
