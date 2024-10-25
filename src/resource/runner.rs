use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use anyhow::Result;
use binrw::BinRead;
use log::debug;

use super::data::DataRecord;
use super::names::NameRecord;
use super::tree::{Flags, RecordType, TreeRecord, RECORD_SIZE};
use crate::handlers::handler::Handler;

pub struct Runner {
    tree: u64,
    names: u64,
    data: u64,
    handlers: Vec<Box<dyn Handler>>,
}

impl Runner {
    pub fn new(tree: u64, names: u64, data: u64) -> Self {
        Runner {
            tree,
            names,
            data,
            handlers: Vec::new(),
        }
    }

    pub fn run<R: Read + Seek>(&mut self, reader: &mut R) -> Result<()>{
        for handler in &mut self.handlers {
            handler.handle_before()?;
        }

        self.traverse_tree(reader, 0, Path::new(""))?;

        for handler in &mut self.handlers {
            handler.handle_after()?;
        }
        Ok(())
    }

    pub fn attach_handler(&mut self, handler: Box<dyn Handler>) {
        self.handlers.push(handler);
    }

    fn traverse_tree<R: Read + Seek>(&mut self, reader: &mut R, offset: u64, base_path: &Path) -> Result<()> {
        reader.seek(SeekFrom::Start(self.tree + offset))?;
        let record = TreeRecord::read(reader)?;

        let name = if offset > 0 {
            &self.read_name(reader, record.name_offset)?
        } else {
            ""
        };

        let path = base_path.join(name);

        match &record.variant {
            RecordType::File(_file_record) => {
                let data = self.read_data(reader, record.offset, &record.flags)?;

                debug!(
                    "processing file {:?}, {:?} (size: {:?})",
                    path,
                    record.flags,
                    data.len()
                );

                for handler in &mut self.handlers {
                    handler.handle_file(&path, &data, record.last_modified)?;
                }
            }

            RecordType::Dir(dir_record) => {
                if offset > 0 {
                    debug!("processing dir  {:?}, {:?}", path, record.flags);

                    for handler in &mut self.handlers {
                        handler.handle_dir(&path)?;
                    }
                }

                for i in 0..dir_record.child_count {
                    self.traverse_tree(
                        reader,
                        (record.offset as u64 + i as u64) * RECORD_SIZE,
                        &path,
                    )?;
                }
            }
        }

        Ok(())
    }

    fn read_name<R: Read + Seek>(&self, reader: &mut R, offset: u32) -> Result<String> {
        reader
            .seek(SeekFrom::Start(self.names + offset as u64))?;

        let name_record = NameRecord::read(reader)?;

        Ok(name_record.name()?)
    }

    fn read_data<R: Read + Seek>(&self, reader: &mut R, offset: u32, flags: &Flags) -> Result<Vec<u8>> {
        reader
            .seek(SeekFrom::Start(self.data + offset as u64))?;

        let data_record = DataRecord::read(reader)?;

        match flags {
            Flags::CompressedZlib => data_record.decompress_zlib(),
            Flags::CompressedZstd => data_record.decompress_zstd(),
            _ => Ok(data_record.data_ref().to_owned()),
        }
    }
}
