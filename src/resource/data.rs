use std::io::Read;

use anyhow::Result;
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

    pub fn decompress_zlib(&self) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        let mut decoder = ZlibDecoder::new(&self.data[4..]);
        decoder.read_to_end(&mut data)?;
        Ok(data)
    }

    pub fn decompress_zstd(&self) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        copy_decode(&self.data[..], &mut data)?;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::BinRead;
    use flate2::{Compression, write::ZlibEncoder};
    use std::io::{Cursor, Write};
    use zstd::stream::copy_encode;

    fn encode_data_record(payload: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4 + payload.len());
        bytes.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        bytes.extend_from_slice(payload);
        bytes
    }

    fn zlib_payload(plain: &[u8]) -> Vec<u8> {
        let mut payload = (plain.len() as u32).to_be_bytes().to_vec();
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(plain).unwrap();
        payload.extend(encoder.finish().unwrap());
        payload
    }

    fn zstd_payload(plain: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        copy_encode(plain, &mut out, 0).unwrap();
        out
    }

    #[test]
    fn parses_uncompressed_payload() {
        let plain = b"Hello, World!";
        let bytes = encode_data_record(plain);
        let rec = DataRecord::read(&mut Cursor::new(bytes)).unwrap();
        assert_eq!(rec.data_ref().as_slice(), plain);
    }

    #[test]
    fn decompresses_zlib_payload() {
        let plain = b"the quick brown fox jumps over the lazy dog";
        let bytes = encode_data_record(&zlib_payload(plain));
        let rec = DataRecord::read(&mut Cursor::new(bytes)).unwrap();
        assert_eq!(rec.decompress_zlib().unwrap(), plain);
    }

    #[test]
    fn decompresses_zstd_payload() {
        let plain = b"the quick brown fox jumps over the lazy dog";
        let bytes = encode_data_record(&zstd_payload(plain));
        let rec = DataRecord::read(&mut Cursor::new(bytes)).unwrap();
        assert_eq!(rec.decompress_zstd().unwrap(), plain);
    }

    #[test]
    fn zlib_round_trip_preserves_binary_payload() {
        let plain: Vec<u8> = (0u8..=255).collect();
        let bytes = encode_data_record(&zlib_payload(&plain));
        let rec = DataRecord::read(&mut Cursor::new(bytes)).unwrap();
        assert_eq!(rec.decompress_zlib().unwrap(), plain);
    }
}
