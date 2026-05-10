use binrw::binread;

pub const RECORD_SIZE: u64 = 22u64;

#[derive(Debug)]
#[binread]
#[br(repr(i16))]
pub enum Flags {
    Uncompressed = 0x00,
    CompressedZlib = 0x01,
    Directory = 0x02,
    CompressedZstd = 0x04,
}

impl Flags {
    fn is_dir(&self) -> bool {
        matches!(self, Flags::Directory)
    }

    fn is_file(&self) -> bool {
        !matches!(self, Flags::Directory)
    }
}

#[derive(Debug)]
#[binread]
pub struct FileRecord {
    pub territory: i16,
    pub language: i16,
}

#[derive(Debug)]
#[binread]
pub struct DirRecord {
    pub child_count: u32,
}

#[derive(Debug)]
#[binread]
#[br(import { flags: &Flags })]
pub enum RecordType {
    #[br(pre_assert(flags.is_file()))]
    File(FileRecord),

    #[br(pre_assert(flags.is_dir()))]
    Dir(DirRecord),
}

#[derive(Debug)]
#[binread]
#[br(big)]
pub struct TreeRecord {
    pub name_offset: u32,
    pub flags: Flags,
    #[br(args { flags: &flags })]
    pub variant: RecordType,
    pub offset: u32,
    pub last_modified: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::BinRead;
    use std::io::Cursor;

    fn build_dir_record(child_count: u32, first_child_index: u32) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(RECORD_SIZE as usize);
        bytes.extend_from_slice(&0u32.to_be_bytes());
        bytes.extend_from_slice(&2i16.to_be_bytes());
        bytes.extend_from_slice(&child_count.to_be_bytes());
        bytes.extend_from_slice(&first_child_index.to_be_bytes());
        bytes.extend_from_slice(&0xCAFEBABEu64.to_be_bytes());
        bytes
    }

    fn build_file_record(flags: i16) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(RECORD_SIZE as usize);
        bytes.extend_from_slice(&12u32.to_be_bytes());
        bytes.extend_from_slice(&flags.to_be_bytes());
        bytes.extend_from_slice(&0i16.to_be_bytes());
        bytes.extend_from_slice(&1i16.to_be_bytes());
        bytes.extend_from_slice(&34u32.to_be_bytes());
        bytes.extend_from_slice(&42u64.to_be_bytes());
        bytes
    }

    #[test]
    fn record_size_is_22_bytes() {
        assert_eq!(RECORD_SIZE, 22);
        assert_eq!(build_dir_record(0, 0).len(), RECORD_SIZE as usize);
        assert_eq!(build_file_record(0).len(), RECORD_SIZE as usize);
    }

    #[test]
    fn parses_directory_record() {
        let bytes = build_dir_record(3, 5);
        let rec = TreeRecord::read(&mut Cursor::new(bytes)).unwrap();

        assert_eq!(rec.name_offset, 0);
        assert!(matches!(rec.flags, Flags::Directory));
        assert_eq!(rec.offset, 5);
        assert_eq!(rec.last_modified, 0xCAFEBABE);
        match rec.variant {
            RecordType::Dir(d) => assert_eq!(d.child_count, 3),
            RecordType::File(_) => panic!("expected directory variant"),
        }
    }

    #[test]
    fn parses_uncompressed_file_record() {
        let bytes = build_file_record(0x00);
        let rec = TreeRecord::read(&mut Cursor::new(bytes)).unwrap();

        assert_eq!(rec.name_offset, 12);
        assert!(matches!(rec.flags, Flags::Uncompressed));
        assert_eq!(rec.offset, 34);
        assert_eq!(rec.last_modified, 42);
        match rec.variant {
            RecordType::File(f) => {
                assert_eq!(f.territory, 0);
                assert_eq!(f.language, 1);
            }
            RecordType::Dir(_) => panic!("expected file variant"),
        }
    }

    #[test]
    fn parses_zlib_compressed_flag() {
        let bytes = build_file_record(0x01);
        let rec = TreeRecord::read(&mut Cursor::new(bytes)).unwrap();
        assert!(matches!(rec.flags, Flags::CompressedZlib));
    }

    #[test]
    fn parses_zstd_compressed_flag() {
        let bytes = build_file_record(0x04);
        let rec = TreeRecord::read(&mut Cursor::new(bytes)).unwrap();
        assert!(matches!(rec.flags, Flags::CompressedZstd));
    }

    #[test]
    fn rejects_unknown_flag_value() {
        let bytes = build_file_record(0x7F);
        assert!(TreeRecord::read(&mut Cursor::new(bytes)).is_err());
    }
}
