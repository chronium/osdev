pub trait Schema {
    fn schema_id(&self) -> SchemaId;
}

pub struct SchemaId(u64);
