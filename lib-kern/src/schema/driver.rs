use super::{file::File, map::SchemaMap, FileId, FileType, Schema, SchemaError};
use alloc::{string::String, sync::Arc, vec::Vec};
use spinning::{Mutex, MutexGuard};

pub struct SchemaDriver {
    _inner: Arc<Mutex<SchemaMap>>,
}

impl SchemaDriver {
    pub fn new() -> Self {
        Self {
            _inner: Arc::new(Mutex::new(SchemaMap::new())),
        }
    }
    pub fn register<S: Schema + Sync + Send + 'static>(
        &self,
        name: String,
        schema: S,
    ) -> Result<(), SchemaError> {
        self._inner.lock().register(name, schema)
    }

    pub fn find(&self, path: &str) -> Result<FileType, SchemaError> {
        self._inner.lock().find(path)
    }

    pub fn open(&self, path: &str) -> Result<File, SchemaError> {
        Ok(File {
            fid: self._inner.lock().open(path)?,
            schema: Arc::downgrade(&self._inner),
        })
    }

    pub fn close(&self, fid: &FileId) -> Result<FileId, SchemaError> {
        self._inner.lock().close(fid)
    }

    pub fn read_to_end(&self, fid: &FileId, buf: &mut Vec<u8>) -> Result<usize, SchemaError> {
        self._inner.lock().read_to_end(fid, buf)
    }

    pub fn read_to_string(&self, fid: &FileId, buf: &mut String) -> Result<usize, SchemaError> {
        self._inner.lock().read_to_string(fid, buf)
    }

    pub fn inner(&self) -> MutexGuard<SchemaMap> {
        self._inner.lock()
    }
}
