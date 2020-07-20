use lib_kern::schema::{FileError, FileId, FileResult, FileType, Schema, SchemaId};

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use hashbrown::HashMap;

pub struct SysSchema {
    schema_id: Option<SchemaId>,
    sysinfo: HashMap<String, String>,
    open_files: HashMap<String, FileId>,
    open_paths: HashMap<FileId, String>,
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
        } else if self.open_files.contains_key(path) {
            Err(FileError::AlreadyOpen)
        } else {
            self.open_files.insert(path.clone(), fid);
            self.open_paths.insert(fid, path.clone());
            Ok(fid)
        }
    }

    fn close(&mut self, fid: &FileId) -> FileResult {
        if !self.open_paths.contains_key(fid) {
            Err(FileError::NotFound)
        } else {
            let spath = self.open_paths.remove(fid).unwrap();
            self.open_files.remove(&spath);
            Ok(*fid)
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
            open_files: HashMap::new(),
            open_paths: HashMap::new(),
        }
    }
}
