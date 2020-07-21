use super::{map::SchemaMap, FileId, SchemaError};
use alloc::{fmt, string::String, sync::Weak, vec::Vec};
use spinning::Mutex;

pub struct File {
    pub fid: FileId,
    pub(super) schema: Weak<Mutex<SchemaMap>>,
}

impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("File").field("fid", &self.fid).finish()
    }
}

impl File {
    pub fn close(&self) -> Result<FileId, SchemaError> {
        Weak::upgrade(&self.schema).unwrap().lock().close(&self.fid)
    }

    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> Result<usize, SchemaError> {
        Weak::upgrade(&self.schema)
            .unwrap()
            .lock()
            .read_to_end(&self.fid, buf)
    }

    pub fn read_to_string(&self, buf: &mut String) -> Result<usize, SchemaError> {
        Weak::upgrade(&self.schema)
            .unwrap()
            .lock()
            .read_to_string(&self.fid, buf)
    }
}

impl Drop for File {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
