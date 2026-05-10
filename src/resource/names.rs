use anyhow::Result;
use binrw::binread;

#[derive(Debug)]
#[binread]
#[br(big)]
pub struct NameRecord {
    // jump past hash (https://codebrowser.dev/qt6/qtbase/src/corelib/io/qresource.cpp.html#752)
    #[br(pad_after = 4)]
    _length: u16,

    #[br(count = _length)]
    name: Vec<u16>,
}

impl NameRecord {
    pub fn name(&self) -> Result<String> {
        Ok(String::from_utf16(&self.name)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::BinRead;
    use std::io::Cursor;

    fn build_name_record(name: &str) -> Vec<u8> {
        let utf16: Vec<u16> = name.encode_utf16().collect();
        let mut bytes = Vec::with_capacity(2 + 4 + utf16.len() * 2);
        bytes.extend_from_slice(&(utf16.len() as u16).to_be_bytes());
        bytes.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
        for code_unit in utf16 {
            bytes.extend_from_slice(&code_unit.to_be_bytes());
        }
        bytes
    }

    #[test]
    fn parses_ascii_name() {
        let bytes = build_name_record("hello.txt");
        let rec = NameRecord::read(&mut Cursor::new(bytes)).unwrap();
        assert_eq!(rec.name().unwrap(), "hello.txt");
    }

    #[test]
    fn parses_non_ascii_name() {
        let bytes = build_name_record("zażółć.png");
        let rec = NameRecord::read(&mut Cursor::new(bytes)).unwrap();
        assert_eq!(rec.name().unwrap(), "zażółć.png");
    }

    #[test]
    fn parses_empty_name() {
        let bytes = build_name_record("");
        let rec = NameRecord::read(&mut Cursor::new(bytes)).unwrap();
        assert_eq!(rec.name().unwrap(), "");
    }

    #[test]
    fn skips_hash_padding() {
        let mut bytes = build_name_record("a");
        bytes[2..6].copy_from_slice(&[0x12, 0x34, 0x56, 0x78]);
        let rec = NameRecord::read(&mut Cursor::new(bytes)).unwrap();
        assert_eq!(rec.name().unwrap(), "a");
    }
}
