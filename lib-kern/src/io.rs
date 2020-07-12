use alloc::{boxed::Box, collections::BTreeMap, vec::Vec};
use spinning::{Mutex, MutexGuard};

pub trait CharDevice {
    fn write_u8(&mut self, val: u8);
    fn write_str(&mut self, val: &str);

    fn get_rw(&self) -> ReadWrite;
}

pub trait BlockDevice {}

#[derive(Debug)]
pub enum ReadWrite {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

pub struct DeviceMap {
    next_device: u16,
    dev_handles: BTreeMap<&'static str, u16>,
    char_dev_list: BTreeMap<u16, Mutex<Box<dyn CharDevice + Sync + Send>>>,
}

impl DeviceMap {
    pub fn new() -> Self {
        Self {
            next_device: 0,
            dev_handles: BTreeMap::new(),
            char_dev_list: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, name: &'static str, device: impl CharDevice + Sync + Send + 'static) {
        self.dev_handles.insert(name, self.next_device);
        self.char_dev_list
            .insert(self.next_device, Mutex::new(box device));
        self.next_device += 1;
    }

    pub fn get(
        &mut self,
        name: &'static str,
    ) -> Option<MutexGuard<Box<dyn CharDevice + Sync + Send>>> {
        let handle = self.dev_handles.get(name)?;
        Some(self.char_dev_list.get_mut(handle)?.lock())
    }

    pub fn dump_names(&self) -> Vec<&&'static str> {
        self.dev_handles.keys().collect()
    }
}
