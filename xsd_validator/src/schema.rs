use libxml2_rs::{ xmlSchemaFree, xmlSchemaPtr };

pub(crate) struct Schema(xmlSchemaPtr);

impl Schema {
    pub(crate) fn new(schema_ptr: xmlSchemaPtr) -> Self {
        Schema(schema_ptr)
    }
}

impl Drop for Schema {
    fn drop(&mut self) {
        unsafe { xmlSchemaFree(self.0) }
    }
}
