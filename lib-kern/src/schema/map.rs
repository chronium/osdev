use super::{split_schema, FileError, FileId, FileType, Schema, SchemaError, SchemaId};
use hashbrown::HashMap;
use spinning::Mutex;

use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};

pub struct SchemaMap {
    schema_names: HashMap<String, SchemaId>,
    schema_handles: HashMap<SchemaId, Mutex<Box<dyn Schema + Sync + Send>>>,
    next_schema: usize,
    open_files: HashMap<String, FileId>,
    open_paths: HashMap<FileId, String>,
    fid_schema: HashMap<FileId, SchemaId>,
    next_fid: usize,
}

impl SchemaMap {
    pub(super) fn new() -> Self {
        Self {
            schema_names: HashMap::new(),
            schema_handles: HashMap::new(),
            next_schema: 0,
            open_files: HashMap::new(),
            open_paths: HashMap::new(),
            fid_schema: HashMap::new(),
            next_fid: 0,
        }
    }

    pub fn register(
        &mut self,
        name: String,
        mut schema: impl Schema + Sync + Send + 'static,
    ) -> Result<(), SchemaError> {
        if self.schema_names.contains_key(&name) {
            return Err(SchemaError::SameNameRegistered(name));
        }

        let schema_id = SchemaId(self.next_schema);

        schema.register(SchemaId(self.next_schema));

        self.schema_names.insert(name, schema_id);
        self.schema_handles
            .insert(schema_id, Mutex::new(box schema));

        self.next_schema += 1;
        Ok(())
    }

    pub fn find(&self, path: &str) -> Result<FileType, SchemaError> {
        let (schema, rest) = split_schema(path);

        if !self.schema_names.contains_key(&schema) {
            return Err(SchemaError::NoSchema(schema));
        }

        let handle = self.schema_names[&schema];
        let schema = &self.schema_handles[&handle];

        schema.lock().find(&rest).ok_or(SchemaError::NotFound(rest))
    }

    pub fn open(&mut self, path: &str) -> Result<FileId, SchemaError> {
        let spath = path.to_string();
        if self.open_files.contains_key(&spath) {
            return Err(SchemaError::AlreadyOpen(spath));
        }

        match self.find(path) {
            Ok(_) => {
                let (schema, rest) = split_schema(path);

                if !self.schema_names.contains_key(&schema) {
                    return Err(SchemaError::NoSchema(schema));
                }

                let handle = self.schema_names[&schema];
                let schema = &self.schema_handles[&handle];

                match schema.lock().open(&rest, FileId(self.next_fid)) {
                    Err(FileError::NotFound) => Err(SchemaError::NotFound(rest)),
                    Err(FileError::AlreadyOpen) => Err(SchemaError::AlreadyOpen(spath)),
                    Ok(fid) => {
                        self.next_fid += 1;
                        self.open_files.insert(spath.clone(), fid);
                        self.open_paths.insert(fid, spath);
                        self.fid_schema.insert(fid, handle);

                        Ok(fid)
                    }
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn close(&mut self, fid: &FileId) -> Result<FileId, SchemaError> {
        if !self.open_paths.contains_key(fid) {
            return Err(SchemaError::NotOpen(*fid));
        }

        let handle = self.fid_schema[fid];
        let schema = &self.schema_handles[&handle];

        match schema.lock().close(fid) {
            Ok(fid) => {
                let spath = self.open_paths.remove(&fid).unwrap();
                self.fid_schema.remove(&fid);
                self.open_files.remove(&spath);

                Ok(fid)
            }
            Err(FileError::NotFound) => Err(SchemaError::NotOpen(*fid)),
            Err(FileError::AlreadyOpen) => unreachable!(),
        }
    }

    pub fn read(&self, fid: &FileId, buf: &mut Vec<u8>) -> Result<usize, SchemaError> {
        if !self.open_paths.contains_key(fid) {
            return Err(SchemaError::NotOpen(*fid));
        }

        let handle = self.fid_schema[fid];
        let schema = &self.schema_handles[&handle];

        schema
            .lock()
            .read(fid, buf)
            .or(Err(SchemaError::NoRead(*fid)))
    }

    pub fn dump_names(&self) -> Vec<&String> {
        self.schema_names.keys().collect()
    }
}
