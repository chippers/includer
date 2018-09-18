#[derive(Debug)]
pub struct WebAsset {
    pub uri: &'static str,
    pub data: &'static [u8],
    pub data_gz: Option<&'static [u8]>,
    pub data_br: Option<&'static [u8]>,
}

impl WebAsset {
    pub fn uri(&self) -> &'static str {
        self.uri
    }

    pub fn data(&self) -> &'static [u8] {
        self.data
    }

    pub fn data_gz(&self) -> Option<&'static [u8]> {
        self.data_gz
    }

    pub fn data_br(&self) -> Option<&'static [u8]> {
        self.data_br
    }
}
