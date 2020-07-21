use lib_kern::schema::{FileError, FileId, FileResult, FileType, Schema, SchemaId};

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use hashbrown::HashMap;

pub struct SysSchema {
    schema_id: Option<SchemaId>,
    sysinfo: HashMap<String, String>,
    by_path: HashMap<String, FileId>,
    by_fid: HashMap<FileId, String>,
}

impl Schema for SysSchema {
    fn schema_id(&self) -> SchemaId {
        self.schema_id.unwrap()
    }

    fn register(&mut self, id: SchemaId) {
        if self.schema_id.is_some() {
            panic!("Sys schema already registered");
        }

        self.schema_id = Some(id);
    }

    fn find(&self, path: &String) -> Option<FileType> {
        self.sysinfo.get(path).map(|_| FileType::File)
    }

    fn open(&mut self, path: &String, fid: FileId) -> FileResult {
        if !self.sysinfo.contains_key(path) {
            Err(FileError::NotFound)
        } else if self.by_path.contains_key(path) {
            Err(FileError::AlreadyOpen)
        } else {
            self.by_path.insert(path.clone(), fid);
            self.by_fid.insert(fid, path.clone());
            Ok(fid)
        }
    }

    fn close(&mut self, fid: &FileId) -> FileResult {
        if !self.by_fid.contains_key(fid) {
            Err(FileError::NotFound)
        } else {
            let spath = self.by_fid.remove(fid).unwrap();
            self.by_path.remove(&spath);
            Ok(*fid)
        }
    }

    fn read_to_end(&self, fid: &FileId, buf: &mut Vec<u8>) -> Result<usize, FileError> {
        if !self.by_fid.contains_key(fid) {
            Err(FileError::NotFound)
        } else {
            let val = &self.sysinfo[&self.by_fid[fid]];
            buf.extend_from_slice(&val.as_bytes()[..]);
            Ok(val.len())
        }
    }

    fn read_to_string(&self, fid: &FileId, buf: &mut String) -> Result<usize, FileError> {
        if !self.by_fid.contains_key(fid) {
            Err(FileError::NotFound)
        } else {
            buf.clone_from(&self.sysinfo[&self.by_fid[fid]]);
            Ok(buf.len())
        }
    }
}

impl SysSchema {
    pub fn new() -> Self {
        let mut sysinfo = HashMap::new();
        sysinfo.insert("info".to_string(), "Hello World".to_string());

        Self {
            schema_id: None,
            sysinfo,
            by_path: HashMap::new(),
            by_fid: HashMap::new(),
        }
    }
}
