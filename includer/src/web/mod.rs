#[derive(Debug)]
pub struct WebAsset {
    pub uri: &'static str,
    pub data: &'static [u8],
    pub data_gz: Option<&'static [u8]>,
    pub data_br: Option<&'static [u8]>,
    pub mime: &'static str,
}
