use lib_kern::schema::{Schema, SchemaId};

pub struct SysSchema {
    schema_id: Option<SchemaId>,
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
}

impl SysSchema {
    pub fn new() -> Self {
        Self { schema_id: None }
    }
}
