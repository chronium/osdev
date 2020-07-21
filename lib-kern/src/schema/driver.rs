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
    pub fn register(
        &mut self,
        name: String,
        schema: impl Schema + Sync + Send + 'static,
    ) -> Result<(), SchemaError> {
        self._inner.lock().register(name, schema)
    }

    pub fn find(&self, path: &str) -> Result<FileType, SchemaError> {
        self._inner.lock().find(path)
    }

    pub fn open(&mut self, path: &str) -> Result<File, SchemaError> {
        Ok(File {
            fid: self._inner.lock().open(path)?,
            schema: Arc::downgrade(&self._inner),
        })
    }

    pub fn close(&mut self, fid: &FileId) -> Result<FileId, SchemaError> {
        self._inner.lock().close(fid)
    }

    pub fn read(&self, fid: &FileId, buf: &mut Vec<u8>) -> Result<usize, SchemaError> {
        self._inner.lock().read(fid, buf)
    }

    pub fn inner(&self) -> MutexGuard<SchemaMap> {
        self._inner.lock()
    }
}
