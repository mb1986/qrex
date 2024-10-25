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
