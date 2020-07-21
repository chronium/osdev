use super::{map::SchemaMap, FileId, SchemaError};
use alloc::{fmt, sync::Weak, vec::Vec};
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

    pub fn read(&self, buf: &mut Vec<u8>) -> Result<usize, SchemaError> {
        Weak::upgrade(&self.schema)
            .unwrap()
            .lock()
            .read(&self.fid, buf)
    }
}
