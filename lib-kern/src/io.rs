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
    dev_names: BTreeMap<&'static str, u16>,
    char_dev_handles: BTreeMap<u16, Mutex<Box<dyn CharDevice + Sync + Send>>>,
}

impl DeviceMap {
    pub fn new() -> Self {
        Self {
            next_device: 0,
            dev_names: BTreeMap::new(),
            char_dev_handles: BTreeMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        name: &'static str,
        device: impl CharDevice + Sync + Send + 'static,
    ) -> Result<(), ()> {
        if self.dev_names.contains_key(name) {
            return Err(());
        }

        self.dev_names.insert(name, self.next_device);
        self.char_dev_handles
            .insert(self.next_device, Mutex::new(box device));
        self.next_device += 1;
        Ok(())
    }

    pub fn get(
        &mut self,
        name: &'static str,
    ) -> Option<MutexGuard<Box<dyn CharDevice + Sync + Send>>> {
        let handle = self.dev_names.get(name)?;
        Some(self.char_dev_handles.get_mut(handle)?.lock())
    }

    pub fn dump_names(&self) -> Vec<&&'static str> {
        self.dev_names.keys().collect()
    }
}
