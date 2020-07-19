use hashbrown::HashMap;

use alloc::{boxed::Box, vec::Vec};
use spinning::Mutex;

pub trait Schema {
    fn schema_id(&self) -> SchemaId;
    fn register(&mut self, id: SchemaId);
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct SchemaId(usize);

pub struct SchemaMap {
    schema_names: HashMap<&'static str, SchemaId>,
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
        name: &'static str,
        mut schema: impl Schema + Sync + Send + 'static,
    ) -> Result<(), SchemaError> {
        if self.schema_names.contains_key(name) {
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

    pub fn dump_names(&self) -> Vec<&&'static str> {
        self.schema_names.keys().collect()
    }
}

pub enum SchemaError {
    SameNameRegistered(&'static str),
}
