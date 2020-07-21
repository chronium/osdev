use alloc::{
    string::{String, ToString},
    vec::Vec,
};

pub mod driver;
pub mod file;
pub mod map;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct FileId(usize);
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct SchemaId(usize);

pub type FileResult = Result<FileId, FileError>;

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

pub trait Schema {
    fn schema_id(&self) -> SchemaId;
    fn register(&mut self, id: SchemaId);

    fn find(&self, path: &String) -> Option<FileType>;

    fn open(&mut self, path: &String, fid: FileId) -> FileResult;
    fn close(&mut self, fid: &FileId) -> FileResult;
    fn read(&self, fid: &FileId, buf: &mut Vec<u8>) -> Result<usize, FileError>;
}

pub(self) fn split_schema(path: &str) -> (String, String) {
    if let &[schema, rest] = path.split(":").collect::<Vec<_>>().as_slice() {
        (
            schema.to_string(),
            rest.trim_start_matches("//").to_string(),
        )
    } else {
        unreachable!();
    }
}

#[derive(Debug)]
pub enum SchemaError {
    SameNameRegistered(String),
    NoSchema(String),
    NotFound(String),
    AlreadyOpen(String),
    NotOpen(FileId),
    NoRead(FileId),
}
