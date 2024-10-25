use std::io::Read;

use binrw::binread;
use flate2::read::ZlibDecoder;
use zstd::stream::copy_decode;

#[derive(Debug)]
#[binread]
#[br(big)]
pub struct DataRecord {
    _size: u32,

    #[br(count = _size)]
    data: Vec<u8>,
}

impl DataRecord {
    pub fn data_ref(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn decompress_zlib(&self) -> Vec<u8> {
        let mut data = Vec::new();
        let mut decoder = ZlibDecoder::new(&self.data[4..]);
        decoder.read_to_end(&mut data).unwrap();
        data
    }

    pub fn decompress_zstd(&self) -> Vec<u8> {
        let mut data = Vec::new();
        copy_decode(&self.data[..], &mut data).unwrap();
        data
    }
}
