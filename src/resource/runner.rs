use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use anyhow::Result;
use binrw::BinRead;
use log::debug;

use super::data::DataRecord;
use super::names::NameRecord;
use super::tree::{Flags, RECORD_SIZE, RecordType, TreeRecord};
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

    pub fn run<R: Read + Seek>(&mut self, reader: &mut R) -> Result<()> {
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

    fn traverse_tree<R: Read + Seek>(
        &mut self,
        reader: &mut R,
        offset: u64,
        base_path: &Path,
    ) -> Result<()> {
        reader.seek(SeekFrom::Start(self.tree + offset))?;
        let record = TreeRecord::read(reader)?;

        let name = if offset > 0 {
            self.read_name(reader, record.name_offset)?
        } else {
            String::new()
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
        reader.seek(SeekFrom::Start(self.names + offset as u64))?;

        let name_record = NameRecord::read(reader)?;

        name_record.name()
    }

    fn read_data<R: Read + Seek>(
        &self,
        reader: &mut R,
        offset: u32,
        flags: &Flags,
    ) -> Result<Vec<u8>> {
        reader.seek(SeekFrom::Start(self.data + offset as u64))?;

        let data_record = DataRecord::read(reader)?;

        match flags {
            Flags::CompressedZlib => data_record.decompress_zlib(),
            Flags::CompressedZstd => data_record.decompress_zstd(),
            _ => Ok(data_record.data_ref().to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::{Compression, write::ZlibEncoder};
    use std::cell::RefCell;
    use std::io::{Cursor, Write};
    use std::rc::Rc;

    type EventLog = Rc<RefCell<Vec<String>>>;

    struct Recorder {
        events: EventLog,
    }

    impl Handler for Recorder {
        fn handle_before(&mut self) -> Result<()> {
            self.events.borrow_mut().push("before".into());
            Ok(())
        }

        fn handle_after(&mut self) -> Result<()> {
            self.events.borrow_mut().push("after".into());
            Ok(())
        }

        fn handle_dir(&mut self, path: &Path) -> Result<()> {
            self.events
                .borrow_mut()
                .push(format!("dir:{}", path.display()));
            Ok(())
        }

        fn handle_file(&mut self, path: &Path, data: &[u8], last_modified: u64) -> Result<()> {
            self.events.borrow_mut().push(format!(
                "file:{}={}@{}",
                path.display(),
                String::from_utf8_lossy(data),
                last_modified
            ));
            Ok(())
        }
    }

    #[derive(Default)]
    struct Blob {
        tree: Vec<u8>,
        names: Vec<u8>,
        data: Vec<u8>,
    }

    impl Blob {
        fn record_count(&self) -> u32 {
            self.tree.len() as u32 / RECORD_SIZE as u32
        }

        fn add_dir(&mut self, name_offset: u32, child_count: u32, first_child_index: u32) -> u32 {
            let idx = self.record_count();
            self.tree.extend_from_slice(&name_offset.to_be_bytes());
            self.tree.extend_from_slice(&2i16.to_be_bytes());
            self.tree.extend_from_slice(&child_count.to_be_bytes());
            self.tree
                .extend_from_slice(&first_child_index.to_be_bytes());
            self.tree.extend_from_slice(&0u64.to_be_bytes());
            idx
        }

        fn add_file(
            &mut self,
            name_offset: u32,
            flags: i16,
            data_offset: u32,
            last_modified: u64,
        ) -> u32 {
            let idx = self.record_count();
            self.tree.extend_from_slice(&name_offset.to_be_bytes());
            self.tree.extend_from_slice(&flags.to_be_bytes());
            self.tree.extend_from_slice(&0i16.to_be_bytes());
            self.tree.extend_from_slice(&0i16.to_be_bytes());
            self.tree.extend_from_slice(&data_offset.to_be_bytes());
            self.tree.extend_from_slice(&last_modified.to_be_bytes());
            idx
        }

        fn add_name(&mut self, name: &str) -> u32 {
            let offset = self.names.len() as u32;
            let utf16: Vec<u16> = name.encode_utf16().collect();
            self.names
                .extend_from_slice(&(utf16.len() as u16).to_be_bytes());
            self.names.extend_from_slice(&[0u8; 4]);
            for cu in utf16 {
                self.names.extend_from_slice(&cu.to_be_bytes());
            }
            offset
        }

        fn add_data(&mut self, payload: &[u8]) -> u32 {
            let offset = self.data.len() as u32;
            self.data
                .extend_from_slice(&(payload.len() as u32).to_be_bytes());
            self.data.extend_from_slice(payload);
            offset
        }

        fn build(self) -> (Vec<u8>, u64, u64, u64) {
            let tree_addr = 0u64;
            let names_addr = self.tree.len() as u64;
            let data_addr = names_addr + self.names.len() as u64;
            let mut buf = Vec::with_capacity(self.tree.len() + self.names.len() + self.data.len());
            buf.extend(self.tree);
            buf.extend(self.names);
            buf.extend(self.data);
            (buf, tree_addr, names_addr, data_addr)
        }
    }

    fn run(blob: Blob) -> Vec<String> {
        let log: EventLog = Rc::new(RefCell::new(Vec::new()));
        let (bytes, tree, names, data) = blob.build();
        let mut runner = Runner::new(tree, names, data);
        runner.attach_handler(Box::new(Recorder {
            events: log.clone(),
        }));
        runner.run(&mut Cursor::new(bytes)).unwrap();
        log.borrow().clone()
    }

    #[test]
    fn empty_root_dir_emits_only_before_after() {
        let mut blob = Blob::default();
        blob.add_dir(0, 0, 1);
        assert_eq!(run(blob), vec!["before", "after"]);
    }

    #[test]
    fn extracts_single_root_file() {
        let mut blob = Blob::default();
        let name = blob.add_name("hello.txt");
        let data = blob.add_data(b"hi");
        blob.add_dir(0, 1, 1);
        blob.add_file(name, 0x00, data, 1234);

        assert_eq!(run(blob), vec!["before", "file:hello.txt=hi@1234", "after"]);
    }

    #[test]
    fn traverses_nested_directory() {
        // Layout:
        //   0: root  (children 1..2)
        //   1: subdir (children 2..3)
        //   2: file inside subdir
        let mut blob = Blob::default();
        let subdir_name = blob.add_name("sub");
        let file_name = blob.add_name("inner.txt");
        let payload = blob.add_data(b"nested");

        blob.add_dir(0, 1, 1);
        blob.add_dir(subdir_name, 1, 2);
        blob.add_file(file_name, 0x00, payload, 7);

        assert_eq!(
            run(blob),
            vec!["before", "dir:sub", "file:sub/inner.txt=nested@7", "after",]
        );
    }

    #[test]
    fn decompresses_zlib_file_during_traversal() {
        let plain = b"compressed body";
        let mut payload = (plain.len() as u32).to_be_bytes().to_vec();
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(plain).unwrap();
        payload.extend(encoder.finish().unwrap());

        let mut blob = Blob::default();
        let name = blob.add_name("z.bin");
        let data = blob.add_data(&payload);
        blob.add_dir(0, 1, 1);
        blob.add_file(name, 0x01, data, 0);

        assert_eq!(
            run(blob),
            vec!["before", "file:z.bin=compressed body@0", "after"]
        );
    }

    #[test]
    fn dispatches_to_multiple_handlers_in_order() {
        let mut blob = Blob::default();
        let name = blob.add_name("a");
        let data = blob.add_data(b"x");
        blob.add_dir(0, 1, 1);
        blob.add_file(name, 0x00, data, 0);
        let (bytes, tree, names, data_addr) = blob.build();

        let log_a: EventLog = Rc::new(RefCell::new(Vec::new()));
        let log_b: EventLog = Rc::new(RefCell::new(Vec::new()));
        let mut runner = Runner::new(tree, names, data_addr);
        runner.attach_handler(Box::new(Recorder {
            events: log_a.clone(),
        }));
        runner.attach_handler(Box::new(Recorder {
            events: log_b.clone(),
        }));
        runner.run(&mut Cursor::new(bytes)).unwrap();

        assert_eq!(*log_a.borrow(), vec!["before", "file:a=x@0", "after"]);
        assert_eq!(*log_b.borrow(), *log_a.borrow());
    }
}
