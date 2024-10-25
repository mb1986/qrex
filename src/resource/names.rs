use binrw::binread;

use crate::error::Result;

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