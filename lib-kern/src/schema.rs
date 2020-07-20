use hashbrown::HashMap;

use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};
use spinning::Mutex;

#[derive(Debug)]
pub struct FileId(usize);

#[derive(Debug)]
pub enum FileType {
    File,
    Directory,
}

#[derive(Debug)]
pub enum FileError {
    NotFound,
    AlreadyOpen,
}

pub type FileResult = Result<FileId, FileError>;

pub trait Schema {
    fn schema_id(&self) -> SchemaId;
    fn register(&mut self, id: SchemaId);

    fn find(&self, path: &String) -> Option<FileType>;

    fn open(&mut self, path: &String) -> FileResult;
    fn close(&mut self, path: &String) -> FileResult;
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct SchemaId(usize);

pub struct SchemaMap {
    schema_names: HashMap<String, SchemaId>,
    schema_handles: HashMap<SchemaId, Mutex<Box<dyn Schema + Sync + Send>>>,
    next_schema: usize,
}

impl SchemaMap {
    pub fn new() -> Self {
        Self {
            schema_names: HashMap::new(),
            schema_handles: HashMap::new(),
            next_schema: 0,
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
        if let &[schema, rest] = path.split(":").collect::<Vec<_>>().as_slice() {
            let schema = schema.to_string();
            let rest = rest.trim_start_matches("//").to_string();

            if !self.schema_names.contains_key(&schema) {
                return Err(SchemaError::NoSchema(schema));
            }

            let handle = self.schema_names[&schema];
            let schema = &self.schema_handles[&handle];

            schema.lock().find(&rest).ok_or(SchemaError::NotFound(rest))
        } else {
            unreachable!();
        }
    }

    pub fn dump_names(&self) -> Vec<&String> {
        self.schema_names.keys().collect()
    }
}

#[derive(Debug)]
pub enum SchemaError {
    SameNameRegistered(String),
    NoSchema(String),
    NotFound(String),
}
