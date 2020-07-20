use lib_kern::schema::{FileResult, FileType, Schema, SchemaId};

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use hashbrown::HashMap;

pub struct SysSchema {
    schema_id: Option<SchemaId>,
    sysinfo: HashMap<String, String>,
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

    fn open(&mut self, path: &String) -> FileResult {
        todo!()
    }

    fn close(&mut self, path: &String) -> FileResult {
        todo!()
    }
}

impl SysSchema {
    pub fn new() -> Self {
        let mut sysinfo = HashMap::new();
        sysinfo.insert("info".to_string(), "Hello World".to_string());

        Self {
            schema_id: None,
            sysinfo,
        }
    }
}
